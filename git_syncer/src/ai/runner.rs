use super::{action::*, brain::*, diff_parser::*, strategy::*};
use crate::ai::learning::{LearningConfig, LearningState};
use crate::ai::nlp::word_frequencies;
use crate::all_access::context::{get_current_branch, get_git_diff};
use colored::*;

pub fn ai_autopilot() -> bool {
    // Création du dossier sandbox pour les tests IA (projets Rust, workspaces, etc.)
    let _ = std::fs::create_dir_all("git_syncer/sandbox");

    // Activation totale des modules IA
    let config = LearningConfig::default();
    let mut learning_state = LearningState::default();

    let brain_path = "git_syncer/brain/brain.bin";
    let mut brain = CommitBrain::load(brain_path);

    // Injection d'exemples humains depuis tous les .ron dans brain/*
    let brain_root = "git_syncer/brain";
    if let Ok(entries) = std::fs::read_dir(brain_root) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Ok(subentries) = std::fs::read_dir(&path) {
                    for subentry in subentries.flatten() {
                        let subpath = subentry.path();
                        if subpath.extension().map(|e| e == "ron").unwrap_or(false) {
                            brain.inject_examples_ron(subpath.to_str().unwrap());
                        }
                    }
                }
            } else if path.extension().map(|e| e == "ron").unwrap_or(false) {
                brain.inject_examples_ron(path.to_str().unwrap());
            }
        }
    }

    // (1) Analyse du contexte (diff)
    let diff = get_git_diff();
    let (keywords, op_types) = extract_features(&diff);
    let pattern = CommitPattern {
        file_keywords: keywords.clone(),
        op_types: op_types.clone(),
    };

    let current_branch = get_current_branch();

    // (2) Décider la prochaine action (on passe current_branch)
    let action = decide_action(&pattern, &brain, &current_branch);

    // (3) Exécuter l’action
    println!("🤖 AI selected action: {action:?}");
    let output = execute(&action).expect("Erreur exécution git");
    let success = output.status.success();
    println!("{}", String::from_utf8_lossy(&output.stdout));
    if !success {
        eprintln!("{}", String::from_utf8_lossy(&output.stderr).red());
    }

    // (4) Feedback/apprentissage sur le message de commit ou action
    let msg = match &action {
        GitAction::Commit(m) => m.clone(),
        _ => format!("{:?}", action),
    };
    brain.learn_msg(pattern.clone(), msg.clone(), success);

    // (5) Sélection naturelle + mutation
    brain.natural_selection();
    let freq = word_frequencies(&keywords);
    let mut vocab: Vec<String> = freq.iter().map(|(w, _)| w.clone()).collect();
    vocab.sort_by_key(|w| std::cmp::Reverse(*freq.get(w).unwrap_or(&0)));
    brain.mutate(&vocab);

    // (6) Apprentissage sur le succès de l'action (statistiques)
    brain.action_success(&format!("{:?}", action));

    // (7) Persiste tout (le .bin est toujours à jour après chaque run)
    brain.save(brain_path);

    // (8) Utilisation réelle de LearningConfig et LearningState
    // Simule une mutation et log l'apprentissage dans learning_state
    let mutation_desc = format!("Action: {:?}, Msg: {}", action, msg);
    learning_state.log_mutation(
        &mutation_desc,
        success,
        if success {
            None
        } else {
            Some(String::from_utf8_lossy(&output.stderr).to_string())
        },
    );
    learning_state.save("git_syncer/brain/learning_state.bin");
    let _ = LearningState::load("git_syncer/brain/learning_state.bin");
    LearningState::scan_history_and_learn(&mut learning_state, "git_syncer/ia_logs/ia.log");

    // (9) Utilisation de config pour contrôler le comportement (exemple)
    if !config.enable_learning {
        println!("⚠️  L'apprentissage est désactivé dans la config IA.");
    }
    if !config.enable_mutation {
        println!("⚠️  Les mutations sont désactivées dans la config IA.");
    }

    success
}
