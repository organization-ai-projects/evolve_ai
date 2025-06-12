use std::process::Command;

pub fn sync_action(action: &str, branch: &str, base: &str, remote: &str) -> Result<(), String> {
    match action {
        "rebase" => {
            let status = Command::new("git")
                .args(["rebase", base])
                .status()
                .map_err(|e| format!("Échec rebase: {e}"))?;
            if status.success() {
                Ok(())
            } else {
                Err(
                    "Conflits lors du rebase. Résous puis fais 'git rebase --continue'."
                        .to_string(),
                )
            }
        }
        "merge" => {
            let status = Command::new("git")
                .args(["merge", base])
                .status()
                .map_err(|e| format!("Échec merge: {e}"))?;
            if status.success() {
                Ok(())
            } else {
                Err("Conflits lors du merge. Résous puis fais 'git merge --continue'.".to_string())
            }
        }
        "pull-only" => {
            let status = Command::new("git")
                .args(["pull", remote, branch])
                .status()
                .map_err(|e| format!("Échec pull-only: {e}"))?;
            if status.success() {
                Ok(())
            } else {
                Err("Pull échoué.".to_string())
            }
        }
        _ => Err("Action inconnue".to_string()),
    }
}
