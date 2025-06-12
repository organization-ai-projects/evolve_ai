use std::process::{Command, Output};

#[derive(Debug, Clone)]
pub enum GitAction {
    AddAll,
    Commit(String),
    Push,
    Pull,
    Rebase { base: String },
    Merge { branch: String },
    CreatePR { base: String, head: String },
    // Extensible : ajoute ce que tu veux
}

pub fn execute(action: &GitAction) -> std::io::Result<Output> {
    match action {
        GitAction::AddAll => Command::new("git").args(["add", "."]).output(),
        GitAction::Commit(msg) => Command::new("git").args(["commit", "-m", msg]).output(),
        GitAction::Push => Command::new("git").args(["push"]).output(),
        GitAction::Pull => Command::new("git").args(["pull"]).output(),
        GitAction::Rebase { base } => Command::new("git").args(["rebase", base]).output(),
        GitAction::Merge { branch } => Command::new("git").args(["merge", branch]).output(),
        GitAction::CreatePR { base, head } => Command::new("gh")
            .args(["pr", "create", "--fill", "--base", base, "--head", head])
            .output(),
    }
}

/// Commit intelligent : n’enregistre que si le score global s’améliore ou une erreur est corrigée.
/// À appeler après une mutation/test.
pub fn smart_commit(
    learning_state: &crate::ai::learning::LearningState,
    old_score: i32,
    logfile: &Option<String>,
) -> bool {
    let new_score = learning_state.global_score;
    if new_score > old_score {
        // Commit car amélioration détectée
        let msg = format!(
            "smart-commit: amélioration détectée (score {} -> {})",
            old_score, new_score
        );
        crate::git_utils::log_message(logfile, &msg);
        let _ = std::process::Command::new("git")
            .args(["commit", "-am", &msg])
            .status();
        true
    } else {
        // Pas d’amélioration, pas de commit
        crate::git_utils::log_message(logfile, "smart-commit: pas d'amélioration, rollback");
        let _ = std::process::Command::new("git")
            .args(["reset", "--hard"])
            .status();
        false
    }
}
