use crate::agent_config::AgentConfig;
use crate::agent_listing::AgentsListing;
use crate::agent_selection::SelectionCriteria;
use crate::agent_structural_code::AgentStructuralCode;
use crate::agent_validator;
use crate::manage_agents_commands;
use crate::notifications::notifier;
use crate::pipelines::life_cycle::{
    agent_safety, agent_scan_update, genome_sync, natural_selection,
};
use crate::project_paths::ProjectPaths;
use crate::scan_agents::RustScanner;
use rayon::prelude::*;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

pub struct LifeManager {
    listing_path: PathBuf,
}

impl LifeManager {
    pub fn new(listing_path: PathBuf) -> Self {
        Self { listing_path }
    }

    pub fn manage_agents_lifecycle(&self, listing: &mut AgentsListing, paths: &ProjectPaths) {
        let config = AgentConfig::default();
        let criteria = SelectionCriteria::default();
        let processes = Arc::new(Mutex::new(HashMap::new()));
        let agent_file_hashes = Arc::new(Mutex::new(HashMap::new()));
        let agent_file_metrics = Arc::new(Mutex::new(HashMap::new()));

        // Prépare les logs
        std::fs::create_dir_all(&config.log_dir).expect("Failed to create logs dir");

        loop {
            let scanner = RustScanner::new(paths.workspace_dir.clone());

            // Pooling : traite les agents par chunks de 10
            for agent_chunk in listing.agents.chunks_mut(10) {
                agent_chunk.par_iter_mut().for_each(|agent| {
                    if !agent.active {
                        return;
                    }
                    // Ajout : skip si l'agent est en backoff suite à crash
                    if agent.has_recent_crash(config.backoff_delay.as_secs()) {
                        return;
                    }

                    // Utilise des verrous pour accéder aux variables partagées
                    let mut hashes = agent_file_hashes.lock().unwrap();
                    let mut metrics = agent_file_metrics.lock().unwrap();
                    let mut procs = processes.lock().unwrap();

                    // 1. Scan, détection des fichiers modifiés et mise à jour de l'état de l'agent après scan
                    let scan_update = match agent_scan_update::scan_and_update_agent(
                        agent,
                        &scanner,
                        &mut *hashes,
                        &mut *metrics,
                    ) {
                        Some(res) => res,
                        None => return,
                    };
                    let changed_files = scan_update.changed_files;
                    let scan_files = scan_update.scan_files;
                    let code_hash = scan_update.scan_result_code_hash;

                    // Vérifie la sécurité, gère l'état de l'agent si non sûr et met à jour les métriques
                    if !agent_safety::check_and_handle_agent_safety_and_metrics(
                        agent,
                        &scan_files,
                        &changed_files,
                    ) {
                        return;
                    }

                    // Calcule les métriques totales pour l'agent à partir du cache
                    let total_metrics = agent.file_metrics.values().fold(
                        AgentStructuralCode::default(),
                        |mut acc, m| {
                            acc.unwrap_count += m.unwrap_count;
                            acc
                        },
                    );

                    // Validation & Compilation (toujours sur l'agent complet)
                    let is_valid = agent_validator::is_code_valid(&agent.path);
                    agent.is_valid = is_valid;
                    if !is_valid {
                        agent.active = false;
                        notifier::notify_disabled(agent, "Agent invalide (ne compile pas)".into());
                        return;
                    }

                    // Gestion du code_hash (toujours sur l'agent complet)
                    if code_hash != agent.file_hashes.code_hash {
                        // 3. Kill & Restart
                        let mut old_process = procs.remove(&agent.name);
                        match manage_agents_commands::reload_agent(
                            &agent.path,
                            &mut agent.is_running,
                            old_process.as_mut(),
                        ) {
                            Ok(Some(child)) => {
                                procs.insert(agent.name.clone(), child);
                                agent.file_hashes.code_hash = code_hash;
                            }
                            _ => {
                                eprintln!("❌ Échec relance {}", agent.name);
                            }
                        }
                    }

                    // Sélection naturelle
                    natural_selection::process_natural_selection(
                        agent,
                        &mut procs,
                        &total_metrics,
                        &config,
                        &criteria,
                    );

                    // Synchronisation avec le génome
                    genome_sync::sync_agent_with_genome(agent, paths);

                    // Hook d'event mort
                    if !agent.active {
                        crate::notifications::notifier::notify_killed(
                            agent,
                            "Agent désactivé (hook event)".to_string(),
                        );
                    }
                });
            }

            // 5. Monitoring des processus
            processes.lock().unwrap().retain(|name, child| {
                match child.try_wait() {
                    Ok(None) => true, // Toujours en vie
                    _ => {
                        if let Some(agent) = listing.agents.iter_mut().find(|a| &a.name == name) {
                            agent.is_running = false;
                        }
                        false
                    }
                }
            });

            // 6. Sauvegarde de l'état
            if let Err(e) = bincode::serialize(&listing).and_then(|bytes| {
                std::fs::write(&self.listing_path, bytes).map_err(bincode::Error::from)
            }) {
                eprintln!("⚠️ Erreur sauvegarde: {}", e);
            }

            std::thread::sleep(std::time::Duration::from_secs(5));
        }
    }
}
