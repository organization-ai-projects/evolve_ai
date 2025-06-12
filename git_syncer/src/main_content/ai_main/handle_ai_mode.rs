use super::jobs::{get_ia_jobs, IaJobTrait};
use super::log::ia_log_json;
use super::state::IaStateManager;
use crate::cli::Args;
use chrono::Utc;
use rayon::prelude::*;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::{Duration, Instant};

pub fn handle_ai_mode(args: &Args) -> bool {
    // Utilise le gestionnaire d'état pour tous les chemins et la logique state
    let state = std::sync::Arc::new(std::sync::Mutex::new(IaStateManager::new()));
    let ia_log_dir = state.lock().unwrap().log_dir().to_string();

    // Undo IA
    if args.undo_ia {
        if state.lock().unwrap().undo_last_action() {
            println!("✅ Undo IA effectué.");
        } else {
            println!("❌ Aucun commit IA à annuler.");
        }
        return true;
    }

    // Status IA
    if args.status {
        state.lock().unwrap().print_status();
        return true;
    }

    // Self-update
    if args.self_update {
        state.lock().unwrap().self_update();
        return true;
    }

    // IA autonome en boucle avec Rayon
    let running = Arc::new(AtomicBool::new(true));
    {
        let running = running.clone();
        ctrlc::set_handler(move || {
            println!("\n🛑 Arrêt demandé, sauvegarde de l'état...");
            running.store(false, Ordering::SeqCst);
        })
        .expect("Erreur installation handler Ctrl+C");
    }

    if args.start_ia {
        if !state.lock().unwrap().try_lock() {
            eprintln!("❌ IA déjà en cours d'exécution (lock détecté)");
            return true;
        }
        state.lock().unwrap().init_running();

        let running_clone = running.clone();
        let state_thread = state.clone();
        let ia_log_dir = ia_log_dir.to_string();

        std::thread::spawn(move || {
            let start_time = Utc::now();
            let mut cycles = 0;
            let mut success_count = 0;
            let mut fail_count = 0;
            let mut last_cycle: String;

            while running_clone.load(Ordering::SeqCst) {
                let jobs: Vec<Box<dyn IaJobTrait + Send + 'static>> = get_ia_jobs();

                // Utilise Arc<Mutex<Vec<...>>> pour collecter les résultats en parallèle
                use std::sync::Mutex;
                let results: Arc<Mutex<Vec<(String, Result<String, String>, u128)>>> =
                    Arc::new(Mutex::new(Vec::with_capacity(jobs.len())));

                jobs.into_par_iter().for_each(|job| {
                    let log_dir = ia_log_dir.clone();
                    let job_name = job.name().to_string();
                    let start = Instant::now();
                    let res = std::thread::spawn(move || job.run(&log_dir))
                        .join()
                        .unwrap_or_else(|_| Err("panic".to_string()));
                    let duration = start.elapsed().as_millis();
                    let mut results = results.lock().unwrap();
                    results.push((job_name, res, duration));
                });

                // Récupère le Vec depuis le Mutex pour itération
                let results = results.lock().unwrap();

                // --- Appel explicite à handle_cycle pour le state IA ---
                if let Ok(mut state) = state_thread.lock() {
                    // On ne garde que les Result<String, String> pour handle_cycle
                    let just_results: Vec<Result<String, String>> =
                        results.iter().map(|(_, res, _)| res.clone()).collect();
                    state.handle_cycle(&just_results);
                }

                for (name, res, duration) in results.iter() {
                    match &res {
                        Ok(msg) => {
                            ia_log_json(name.as_str(), msg.as_str(), true, &ia_log_dir);
                            success_count += 1;
                            if name == "AutoCommit" {
                                if let Ok(state) = state_thread.lock() {
                                    state.log_undo_commit();
                                }
                            }
                        }
                        Err(e) => {
                            ia_log_json(name.as_str(), e.as_str(), false, &ia_log_dir);
                            fail_count += 1;
                        }
                    }
                    println!("[{}] terminé en {}ms", name, duration);
                }

                cycles += 1;
                last_cycle = Utc::now().to_rfc3339();
                let uptime = (Utc::now() - start_time).num_seconds();
                let state_str = format!(
                    "running\ncycles:{cycles}\nlast:{last_cycle}\nsuccess:{success_count}\nfail:{fail_count}\nuptime:{uptime}\nheartbeat:{}",
                    Utc::now().to_rfc3339()
                );
                if let Ok(state) = state_thread.lock() {
                    std::fs::write(state.state_path, state_str).ok();
                }
                ia_log_json(
                    "cycle",
                    &format!("Cycle {cycles} terminé."),
                    true,
                    &ia_log_dir,
                );
                std::thread::sleep(Duration::from_secs(2));
                // Ajoute ce test juste après la pause pour sortir proprement dès que possible
                if !running_clone.load(Ordering::SeqCst) {
                    break;
                }
            }
            println!("🛑 IA stoppée proprement (thread IA terminé).");
            if let Ok(state) = state_thread.lock() {
                state.finalize();
            }
        });

        // Handler Ctrl+C pour stopper uniquement l'IA (pas le process principal)
        ctrlc::set_handler(move || {
            println!("\n🛑 Arrêt demandé pour l'IA (le projet principal continue)...");
            running.store(false, Ordering::SeqCst);
        })
        .expect("Erreur installation handler Ctrl+C");

        loop {
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    }

    if args.stop_ia {
        state.lock().unwrap().stop();
        return true;
    }

    false
}
