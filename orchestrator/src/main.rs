pub const WORKSPACE_DIR: &str = "evolve_ai"; // Racine du workspace evolve_ai/
pub const ORCHESTRATOR_DIR: &str = "orchestrator"; // Sous-dossier orchestrator
pub const AGENTS_DIR: &str = "agents"; // Sous-dossier agents
pub const TEMPLATE_DIR: &str = "agent_template"; // Sous-dossier template des agents

mod agent_code_analysis;
mod agent_config;
mod agent_file_hashes;
mod agent_listing;
mod agent_needs;
mod agent_sanitizer;
mod agent_selection;
mod agent_structural_code;
mod agent_validator; // Ajout du module validator à la place de updater
mod cargo_commands;
mod genome;
mod genome_sync; // Au lieu de genetic_recombination
mod manage_agents_commands;
mod monitoring;
mod notifications;
mod pipelines;
mod project_paths; // Import du module qui gère les chemins
mod scan_agents; // Ajouter le nouveau module
mod sys_commands;

use crate::pipelines::{initiate_project, LifeManager};
use crate::project_paths::ProjectPaths;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::thread;
use std::time::Duration;

fn main() {
    // Initialisation des chemins via ProjectPaths
    let paths = ProjectPaths::new();

    // Définir le nombre d'agents à créer
    let nb_agents = 3; // Exemple : 3 agents

    // ÉTAPE 1: Initialisation complète du projet
    if let Err(e) = initiate_project(&paths, nb_agents) {
        eprintln!("Erreur lors de l'initialisation du projet : {}", e);
        return;
    }

    // Utilisation du helper pour obtenir le chemin du listing
    let listing_path = paths.agent_listing_path();
    let listing_bytes = match std::fs::read(&listing_path) {
        Ok(bytes) => bytes,
        Err(e) => {
            eprintln!("Erreur lors du chargement du listing : {}", e);
            return;
        }
    };
    let listing: crate::agent_listing::AgentsListing =
        match bincode::deserialize(&listing_bytes) {
            Ok(listing) => listing,
            Err(e) => {
                eprintln!("Erreur de désérialisation du listing : {}", e);
                return;
            }
        };

    let listing = Arc::new(Mutex::new(listing));
    let running = Arc::new(AtomicBool::new(true));
    {
        let running = running.clone();
        let listing_path = listing_path.clone();
        let listing = listing.clone();
        ctrlc::set_handler(move || {
            println!("\n🛑 Arrêt demandé (Ctrl+C). Sauvegarde de l'état des agents...");
            // Sauvegarde du listing (sécurisé)
            if let Ok(listing) = listing.lock() {
                if let Ok(bytes) = bincode::serialize(&*listing) {
                    let _ = std::fs::write(&listing_path, bytes);
                }
            }
            running.store(false, Ordering::SeqCst);
        })
        .expect("Erreur lors de l'installation du handler Ctrl+C");
    }

    // ÉTAPE 2: Lancement du pipeline de gestion
    let life_manager = LifeManager::new(listing_path);

    // Boucle principale avec gestion d'arrêt propre
    while running.load(Ordering::SeqCst) {
        if let Ok(mut listing) = listing.lock() {
            life_manager.manage_agents_lifecycle(&mut *listing, &paths);
        }
        thread::sleep(Duration::from_secs(1));
    }

    println!("✅ Arrêt propre terminé.");
}
