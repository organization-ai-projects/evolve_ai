use super::action::GitAction;
use super::brain::{CommitBrain, CommitPattern};

/// Politique évolutive : l’IA peut choisir toutes les actions selon le contexte
pub fn decide_action(
    pattern: &CommitPattern,
    brain: &CommitBrain,
    current_branch: &str,
) -> GitAction {
    if !pattern.op_types.is_empty() {
        if pattern.op_types.contains(&"add".to_string()) {
            return GitAction::AddAll;
        }
        if pattern.op_types.contains(&"mod".to_string()) {
            return GitAction::Commit(
                brain
                    .predict_msg(pattern)
                    .map(|n| n.message.clone())
                    .unwrap_or_else(|| format!("update: {:?}", pattern.file_keywords)),
            );
        }
        if pattern.op_types.contains(&"remove".to_string()) {
            return GitAction::Pull;
        }
        // Si le mot clé "rebase" est dans les fichiers modifiés, rebase sur main
        if pattern.file_keywords.iter().any(|k| k.contains("rebase")) {
            return GitAction::Rebase {
                base: "main".to_string(),
            };
        }
        // Si le mot clé "merge" est dans les fichiers modifiés, merge main dans la branche courante
        if pattern.file_keywords.iter().any(|k| k.contains("merge")) {
            return GitAction::Merge {
                branch: current_branch.to_string(),
            };
        }
        // Si le mot clé "pr" est dans les fichiers modifiés, crée une PR depuis la branche courante
        if pattern.file_keywords.iter().any(|k| k.contains("pr")) {
            return GitAction::CreatePR {
                base: "main".to_string(),
                head: current_branch.to_string(),
            };
        }
    }
    // Par défaut, push si rien à faire
    GitAction::Push
}
