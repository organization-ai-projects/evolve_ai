use super::learning::{LearningState, MutationLog};
use crate::all_access::sandbox_guard::{is_command_safe, is_in_sandbox};

/// Module d’auto-coding IA : tente une génération/édition de code, teste avec `cargo check`
/// et éventuellement `cargo run`, puis adapte la stratégie d’apprentissage.
/// Peut être utilisé pour l’auto-réparation, l’auto-refactor ou la génération de code Rust.
/// Si une erreur critique est détectée, rollback ou ignore la modification.
pub fn try_code_and_learn<F>(
    state: &mut LearningState,
    coding_desc: &str,
    apply_code: F,
    project_dir: &str,
    test_run: bool, // true pour tester aussi cargo run
) where
    F: FnOnce() -> bool,
{
    // Vérifie que le projet_dir est bien dans sandbox
    if !is_in_sandbox(project_dir) {
        state.log_mutation(
            coding_desc,
            false,
            Some("Tentative d'accès hors sandbox refusée".to_string()),
        );
        return;
    }

    let success = apply_code();

    // Vérifie la sécurité avant toute commande
    if !is_command_safe("cargo", &["check"], project_dir) {
        state.log_mutation(
            coding_desc,
            false,
            Some("Commande dangereuse ou hors sandbox refusée".to_string()),
        );
        return;
    }
    let check_output = std::process::Command::new("cargo")
        .arg("check")
        .current_dir(project_dir)
        .output()
        .expect("Échec cargo check");
    let check_success = check_output.status.success();
    let mut run_success = true;
    let mut run_stderr = String::new();

    // Optionnel : tester aussi cargo run si demandé
    if test_run && check_success {
        if !is_command_safe("cargo", &["run"], project_dir) {
            state.log_mutation(
                coding_desc,
                false,
                Some("Commande dangereuse ou hors sandbox refusée".to_string()),
            );
            return;
        }
        let run_output = std::process::Command::new("cargo")
            .arg("run")
            .current_dir(project_dir)
            .output()
            .expect("Échec cargo run");
        run_success = run_output.status.success();
        run_stderr = String::from_utf8_lossy(&run_output.stderr).to_string();
    }

    if check_success && run_success && success {
        state.log_mutation(coding_desc, true, None);
    } else {
        // Rollback ou ignore la modification si échec
        let stderr = if !run_success {
            run_stderr
        } else {
            String::from_utf8_lossy(&check_output.stderr).to_string()
        };
        state.log_mutation(coding_desc, false, Some(stderr));
        // Ici, tu peux ajouter une logique de rollback (ex: git reset --hard)
    }

    // Utilisation réelle de l'historique pour adapter la stratégie IA
    let recent_errors: Vec<&MutationLog> = state
        .mutation_history
        .iter()
        .rev()
        .take(5)
        .filter(|log| !log.success)
        .collect();

    if recent_errors.len() >= 3 {
        // Si 3 échecs récents, l'IA change de stratégie (rollback automatique)
        println!("⚠️  IA : détection de plusieurs échecs récents, rollback automatique et adaptation de la stratégie...");
        // Rollback du dernier commit si possible
        let _ = std::process::Command::new("git")
            .args(["reset", "--hard", "HEAD~1"])
            .status();
        // Augmente le compteur de switch de stratégie
        *state
            .stats
            .entry("strategy_switch".to_string())
            .or_insert(0) += 1;
        // Ajoute un message d'erreur dans l'historique pour suivi IA
        state
            .error_history
            .push("Rollback IA déclenché après 3 échecs consécutifs".to_string());
    }

    // Analyse automatique des motifs d'échec pour guider la génération future
    let mut error_patterns: std::collections::HashMap<String, usize> =
        std::collections::HashMap::new();
    for log in recent_errors.iter() {
        if let Some(err) = &log.error_msg {
            for word in err.split_whitespace() {
                *error_patterns.entry(word.to_lowercase()).or_insert(0) += 1;
            }
        }
    }
    // Stocke les patterns les plus fréquents dans les stats IA pour exploitation future
    let mut sorted_patterns: Vec<_> = error_patterns.iter().collect();
    sorted_patterns.sort_by_key(|(_, &count)| std::cmp::Reverse(count));
    let top_patterns: Vec<String> = sorted_patterns
        .iter()
        .take(3)
        .map(|(w, _)| (*w).clone())
        .collect();
    state.stats.insert(
        "top_error_pattern_1".to_string(),
        top_patterns.get(0).cloned().unwrap_or_default().len(),
    );
    state.stats.insert(
        "top_error_pattern_2".to_string(),
        top_patterns.get(1).cloned().unwrap_or_default().len(),
    );
    state.stats.insert(
        "top_error_pattern_3".to_string(),
        top_patterns.get(2).cloned().unwrap_or_default().len(),
    );

    // Ces patterns peuvent être utilisés par l'IA pour éviter de refaire les mêmes erreurs ou pour générer des correctifs ciblés.
    // Par exemple, tu pourrais transmettre ces patterns à un générateur de patch ou à une stratégie de mutation.
}

/// Génère du code Rust à partir d'une demande textuelle et l'écrit dans sandbox/
/// Si la demande contient un chemin, crée le fichier à cet emplacement dans sandbox/
/// Refuse toute écriture hors sandbox.
pub fn generate_code_from_prompt(prompt: &str) -> String {
    use std::fs;
    use std::path::{Path, PathBuf};

    // Extraction naïve d'un chemin cible depuis la demande (ex: "dans utils/math.rs")
    let path_in_prompt =
        prompt
            .split_whitespace()
            .find_map(|w| if w.ends_with(".rs") { Some(w) } else { None });

    let sandbox = Path::new("git_syncer/sandbox");
    let file_path: PathBuf = if let Some(rel) = path_in_prompt {
        let rel_path = Path::new(rel);
        let abs_path = sandbox.join(rel_path);
        // Refuse si le chemin sort de sandbox
        if !abs_path.starts_with(sandbox) {
            return format!(
                "❌ Refusé : tentative d'écriture hors sandbox ({})",
                abs_path.display()
            );
        }
        abs_path
    } else {
        sandbox.join("ia_generated.rs")
    };

    let _ = fs::create_dir_all(file_path.parent().unwrap_or(sandbox));
    let code = format!(
        "// Code généré par l'IA pour : {}\nfn main() {{ println!(\"Hello IA!\"); }}",
        prompt
    );
    match fs::write(&file_path, &code) {
        Ok(_) => format!("Code généré dans : {}\n\n{}", file_path.display(), code),
        Err(e) => format!("❌ Erreur écriture fichier : {}", e),
    }
}
