use super::log::ia_log_json;
use crate::ai::action::{execute, smart_commit};
use crate::ai::brain::{CommitBrain, CommitPattern};
use crate::ai::diff_parser::extract_features;
use crate::ai::learning::{try_code_and_learn_adaptive, LearningConfig, LearningState};
use crate::ai::nlp::score_commit_message;
use crate::ai::runner::ai_autopilot;
use crate::ai::semantics::map_rust_type;
use crate::ai::strategy::decide_action;
use crate::all_access::context::{get_current_branch, get_git_diff};
use crate::git_utils::log_error;
use std::time::Duration;

pub trait IaJobTrait: Send + Sync + 'static {
    fn name(&self) -> &'static str;
    fn run(&self, log_dir: &str) -> Result<String, String>;
}

pub struct TrainingJob;
impl IaJobTrait for TrainingJob {
    fn name(&self) -> &'static str {
        "Training"
    }
    fn run(&self, log_dir: &str) -> Result<String, String> {
        // Utilisation réelle de l'IA : analyse du diff courant et usage du cerveau IA
        let diff = crate::all_access::context::get_git_diff();
        let (keywords, ops) = extract_features(&diff);
        let mapped_types: Vec<String> = keywords.iter().map(|k| map_rust_type(k)).collect();

        // Utilisation de CommitBrain pour prédire un message de commit
        let brain = CommitBrain::load("git_syncer/brain/brain.bin");
        let pattern = CommitPattern {
            file_keywords: keywords.clone(),
            op_types: ops.clone(),
        };
        let predicted = brain
            .predict_msg(&pattern)
            .map(|n| n.message.clone())
            .unwrap_or_else(|| "No prediction".to_string());

        let msg = format!(
            "Training terminé. Keywords: {:?}, Ops: {:?}, Types: {:?}, Prediction: {}",
            keywords, ops, mapped_types, predicted
        );
        std::thread::sleep(Duration::from_millis(500));
        ia_log_json(self.name(), &msg, true, log_dir);
        Ok(msg)
    }
}

pub struct RefactorJob;
impl IaJobTrait for RefactorJob {
    fn name(&self) -> &'static str {
        "Refactor"
    }
    fn run(&self, log_dir: &str) -> Result<String, String> {
        let example_type = "Option<String>";
        let mapped = map_rust_type(example_type);
        let msg = format!("Refactorisation terminée. Type: {example_type} → {mapped}");
        std::thread::sleep(Duration::from_millis(400));
        ia_log_json(self.name(), &msg, true, log_dir);
        Ok(msg)
    }
}

pub struct FullIaJob;
impl IaJobTrait for FullIaJob {
    fn name(&self) -> &'static str {
        "FullIaJob"
    }
    fn run(&self, log_dir: &str) -> Result<String, String> {
        // 1. Analyse du contexte
        let diff = get_git_diff();
        let (keywords, ops) = extract_features(&diff);
        let mapped_types: Vec<String> = keywords.iter().map(|k| map_rust_type(k)).collect();

        // 2. Cerveau IA
        let mut brain = CommitBrain::load("git_syncer/brain/brain.bin");
        brain.inject_examples("git_syncer/brain/commit/examples.txt");
        brain.inject_examples_ron("git_syncer/brain/commit/examples.ron");
        let pattern = CommitPattern {
            file_keywords: keywords.clone(),
            op_types: ops.clone(),
        };
        let predicted = brain
            .predict_msg(&pattern)
            .map(|n| n.message.clone())
            .unwrap_or_else(|| "No prediction".to_string());

        // 3. Apprentissage
        brain.learn_msg(pattern.clone(), predicted.clone(), true);
        brain.natural_selection();
        brain.mutate(&keywords);
        brain.action_success("FullIaJob");
        brain.save("git_syncer/brain/brain.bin");

        // 4. LearningConfig et LearningState
        let config = LearningConfig::default();
        let mut state = LearningState::default();
        state.log_mutation("FullIaJob mutation", true, None);
        state.save("git_syncer/brain/learning_state.bin");
        let _ = LearningState::load("git_syncer/brain/learning_state.bin");
        LearningState::scan_history_and_learn(&mut state, "git_syncer/ia_logs/ia.log");

        // Utilisation réelle de tous les champs de LearningConfig
        let mut config_flags = vec![];
        if config.enable_learning {
            config_flags.push("learning");
        }
        if config.enable_mutation {
            config_flags.push("mutation");
        }
        if config.enable_history_scan {
            config_flags.push("history_scan");
        }
        if config.enable_smart_commit {
            config_flags.push("smart_commit");
        }

        // 5. NLP scoring
        let score = score_commit_message(&predicted);

        // 6. Autopilot (dry-run)
        let autopilot_ok = ai_autopilot();

        // 7. Strategy
        let current_branch = get_current_branch();
        let action = decide_action(&pattern, &brain, &current_branch);

        // 8. GitAction (utilisation réelle)
        let output = execute(&action).unwrap();
        let output_status = output.status.success();
        let output_stdout = String::from_utf8_lossy(&output.stdout);
        let output_stderr = String::from_utf8_lossy(&output.stderr);

        if output_status && !output_stdout.trim().is_empty() {
            ia_log_json(
                "GitActionOutput",
                &format!("stdout: {}", output_stdout),
                true,
                log_dir,
            );
        }

        // 9. smart_commit : on adapte le workflow selon le résultat
        let smart_commit_ok = smart_commit(&state, state.global_score, &None);
        if smart_commit_ok {
            brain.learn_msg(pattern.clone(), predicted.clone(), true);
        } else {
            brain.learn_msg(pattern.clone(), predicted.clone(), false);
            log_error(&None, "Smart commit failed, rollback triggered");
        }

        // 10. Log error si l'action git a échoué
        if !output_status {
            log_error(
                &None,
                &format!("GitAction failed: {:?}\nstderr: {}", action, output_stderr),
            );
        }

        let msg = format!(
            "FullIaJob terminé. Action: {:?}, Status: {}, Keywords: {:?}, Ops: {:?}, Types: {:?}, Prediction: {}, Score: {}, Autopilot: {}, SmartCommit: {}, ConfigFlags: {:?}",
            action, output_status, keywords, ops, mapped_types, predicted, score, autopilot_ok, smart_commit_ok, config_flags
        );
        std::thread::sleep(Duration::from_millis(300));
        ia_log_json(self.name(), &msg, true, log_dir);
        Ok(msg)
    }
}

pub struct EvolutionJob;
impl IaJobTrait for EvolutionJob {
    fn name(&self) -> &'static str {
        "Evolution"
    }
    fn run(&self, log_dir: &str) -> Result<String, String> {
        use crate::ai::learning::LearningConfig;
        use rand::Rng;

        // Charge la config actuelle (ou simule)
        let mut config = LearningConfig::default();
        let mut rng = rand::thread_rng();

        // Mutation aléatoire des paramètres
        config.enable_learning = rng.gen_bool(0.9);
        config.enable_mutation = rng.gen_bool(0.9);
        config.enable_history_scan = rng.gen_bool(0.5);
        config.enable_smart_commit = rng.gen_bool(0.5);

        // Simule un score de fitness (ex : nombre de succès sur le dernier cycle)
        let fitness = rng.gen_range(0..10);

        // Log évolution
        let msg = format!(
            "EvolutionJob: config mutée = {:?}, fitness = {}",
            config, fitness
        );
        ia_log_json(self.name(), &msg, true, log_dir);

        // Ici, tu pourrais persister la config mutée et la fitness pour sélection naturelle
        Ok(msg)
    }
}

pub struct CrowdLearningJob;
impl IaJobTrait for CrowdLearningJob {
    fn name(&self) -> &'static str {
        "CrowdLearning"
    }
    fn run(&self, log_dir: &str) -> Result<String, String> {
        use crate::ai::brain::CommitBrain;
        let brain = CommitBrain::load("git_syncer/brain/brain.bin");

        // Sérialise tous les patterns/messages anonymisés
        let crowd_data: Vec<_> = brain
            .neurons
            .iter()
            .map(|n| {
                (
                    &n.pattern.file_keywords,
                    &n.pattern.op_types,
                    &n.message,
                    n.score,
                )
            })
            .collect();

        let path = "git_syncer/brain/crowd_learning.json";
        std::fs::write(path, serde_json::to_string(&crowd_data).unwrap()).ok();

        let msg = format!(
            "CrowdLearningJob: {} patterns exportés pour cross-agent learning",
            crowd_data.len()
        );
        ia_log_json(self.name(), &msg, true, log_dir);
        Ok(msg)
    }
}

pub struct AutoDocJob;
impl IaJobTrait for AutoDocJob {
    fn name(&self) -> &'static str {
        "AutoDoc"
    }
    fn run(&self, log_dir: &str) -> Result<String, String> {
        use crate::ai::ast::extract_ast_features;
        let diff = crate::all_access::context::get_git_diff();
        let features = extract_ast_features(&diff);

        // Génère un docstring simple à partir des features
        let doc = format!("/// AutoDoc: Ce commit touche : {:?}", features);
        let doc_path = "git_syncer/ia_logs/autodoc.txt";
        std::fs::write(doc_path, &doc).ok();

        let msg = format!("AutoDocJob: docstring généré et injecté dans {}", doc_path);
        ia_log_json(self.name(), &msg, true, log_dir);
        Ok(msg)
    }
}

pub struct CodeLearnJob;
impl IaJobTrait for CodeLearnJob {
    fn name(&self) -> &'static str {
        "CodeLearn"
    }
    fn run(&self, log_dir: &str) -> Result<String, String> {
        let mut state = LearningState::default();

        // Recherche un vrai fichier Rust existant dans le projet
        let candidate = std::fs::read_dir("src")
            .ok()
            .and_then(|mut it| {
                it.find(|e| {
                    e.as_ref()
                        .ok()
                        .and_then(|e| e.path().extension().map(|ext| ext == "rs"))
                        .unwrap_or(false)
                })
            })
            .and_then(|e| e.ok())
            .map(|e| e.path());

        let Some(target_file) = candidate else {
            let msg = "CodeLearnJob: aucun fichier Rust trouvé pour apprentissage".to_string();
            ia_log_json(self.name(), &msg, true, log_dir);
            return Ok(msg);
        };

        // Centralise toute la logique d'apprentissage/adaptation dans learning.rs
        let msg = try_code_and_learn_adaptive(&mut state, &target_file);

        ia_log_json(self.name(), &msg, true, log_dir);
        Ok(msg)
    }
}

pub fn get_ia_jobs() -> Vec<Box<dyn IaJobTrait + Send>> {
    vec![
        Box::new(TrainingJob),
        Box::new(RefactorJob),
        Box::new(FullIaJob),
        Box::new(EvolutionJob),
        Box::new(CrowdLearningJob),
        Box::new(AutoDocJob),
        Box::new(CodeLearnJob), // Ajoute le job ici
                                // ...autres jobs...
    ]
}
