//crée le dossier agents/ à la racine du projet
//récupère le contenu de agent_template/ pour créer les agents
//crée le fichier genome.bin dans agents/ pour chaque agent avec les fichiers réellement utilisés par l'agent, structure du fichier dans genome.rs

use crate::genome_sync;
use crate::pipelines::initiate::agent_info::build_agent_info;
use crate::pipelines::initiate::cargo::update_cargo_toml;
use crate::pipelines::initiate::copy::copy_dir_all;
use crate::pipelines::initiate::generate_initial_genome;
use crate::pipelines::initiate::listing::save_agents_listing;
use crate::pipelines::initiate::log::write_initialization_log;
use crate::project_paths::ProjectPaths;
use crate::scan_agents::RustScanner;
use crate::{
    agent_listing::{generate_agent_id, generate_short_uuid, AgentsListing},
    manage_agents_commands,
};
use rayon::prelude::*;
use std::fs;
use std::sync::Arc;

pub fn initiate_project(paths: &ProjectPaths, nb_agents: usize) -> std::io::Result<()> {
    // 1. Créer le dossier agents/ s'il n'existe pas
    fs::create_dir_all(&paths.agents_dir)?;

    // 2. Vérifier le template
    if !paths.template_dir.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Le dossier agent_template/ est manquant.",
        ));
    }

    // Créer le listing des agents
    let listing = AgentsListing { agents: Vec::new() };
    let initialization_log = Vec::new(); // Log pour récapitulatif

    // Scanne d'abord le template pour connaître sa structure
    let scanner = RustScanner::new(paths.workspace_dir.clone());
    let template_scan_result = match scanner.scan_template(&paths.template_dir) {
        Ok(result) => result,
        Err(e) => {
            eprintln!("❌ Échec scan du template: {}", e);
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Échec scan template",
            ));
        }
    };

    // 3. Créer les agents initiaux avec leur génome (parallélisé)
    let agent_indices: Vec<_> = (1..=nb_agents).collect();
    let listing = Arc::new(parking_lot::Mutex::new(listing));
    let initialization_log = Arc::new(parking_lot::Mutex::new(initialization_log));

    agent_indices.par_chunks(10).for_each(|chunk| {
        chunk.iter().for_each(|_i| {
            // Génération de l'UUID pour l'agent
            let agent_id = generate_agent_id();
            let short_uuid = generate_short_uuid(&agent_id);
            let agent_dir = paths.agents_dir.join(&short_uuid);
            if agent_dir.exists() {
                eprintln!("⚠️ Agent déjà existant : {}", short_uuid);
                return;
            }

            // Copier le template
            if let Err(e) = copy_dir_all(&paths.template_dir, &agent_dir) {
                eprintln!("❌ Échec copie template pour {} : {}", short_uuid, e);
                return; // Gestion douce des erreurs
            }

            // Mettre à jour le Cargo.toml
            if let Err(e) = update_cargo_toml(&agent_dir, &agent_id) {
                eprintln!("❌ Erreur update Cargo.toml pour {} : {}", short_uuid, e);
                return;
            }

            // Générer le génome initial via la fonction utilitaire
            let mut rng = rand::thread_rng();
            let initial_genome = generate_initial_genome(
                &template_scan_result
                    .files
                    .iter()
                    .map(|(p, c)| (p.clone(), c.clone()))
                    .collect::<Vec<_>>(),
                &mut rng,
            );

            // Laisser genome_sync s'occuper de la synchronisation complète
            if let Err(e) = genome_sync::sync_code_with_genome(&initial_genome, &agent_dir) {
                eprintln!(
                    "❌ Échec synchronisation génome pour {} : {}",
                    short_uuid, e
                );
            }

            // Sauvegarder le génome dans le répertoire de l'agent
            let agent_genome_path = agent_dir.join("genome.bin");
            if let Ok(bytes) = bincode::serialize(&initial_genome) {
                let _ = std::fs::write(&agent_genome_path, bytes);
            }

            // Sauvegarder une copie du génome dans le répertoire de l'orchestrateur
            let orchestrator_genome_path = paths.agent_genome_path(&short_uuid);
            let genome_bytes = match bincode::serialize(&initial_genome) {
                Ok(bytes) => bytes,
                Err(e) => {
                    eprintln!("Erreur de sérialisation du génome: {}", e);
                    return;
                }
            };
            if let Err(e) = fs::write(orchestrator_genome_path, genome_bytes) {
                eprintln!("Erreur d'écriture du génome orchestrateur: {}", e);
                return;
            }

            // Scanner l'agent APRÈS synchronisation pour obtenir le hash final
            let scan_result = match scanner.scan_agent(&agent_dir, false) {
                Ok(result) => result,
                Err(e) => {
                    eprintln!("❌ Échec scan pour {} : {}", short_uuid, e);
                    return; // Gestion douce des erreurs
                }
            };

            // Créer AgentInfo avec le scan résultant
            let agent_info = build_agent_info(
                agent_id,
                short_uuid.clone(),
                agent_dir.clone(),
                &scan_result,
            );

            // Hook d'event création
            crate::notifications::notifier::notify_killed(
                &agent_info,
                "Agent créé (hook event)".to_string(),
            );

            // Ajouter l'agent au listing (thread-safe)
            listing.lock().agents.push(agent_info);
            initialization_log
                .lock()
                .push(format!("✅ Agent {} initialisé avec succès", short_uuid));
        });
    });

    let listing = Arc::try_unwrap(listing).unwrap().into_inner();
    let initialization_log = Arc::try_unwrap(initialization_log).unwrap().into_inner();

    // 4. Sauvegarder le listing
    let listing_path = paths.agent_listing_path();
    save_agents_listing(&listing_path, &listing)?;

    // 5. Sauvegarder le log d'initialisation
    let log_path = paths.workspace_dir.join("initialization_log.txt");
    write_initialization_log(&log_path, &initialization_log)?;

    // 6. Lancer tous les agents initiaux en parallèle
    let agent_paths: Vec<_> = listing.agents.iter().map(|a| a.path.clone()).collect();
    let children = manage_agents_commands::run_all_agents(&agent_paths);
    let success_count = children.into_iter().filter(|r| r.is_ok()).count();
    println!("✅ {} agents lancés", success_count);

    Ok(())
}
