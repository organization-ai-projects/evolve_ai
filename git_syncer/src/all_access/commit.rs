use std::process::Command;

pub fn auto_add_and_commit(
    default_commit_msg: &str,
    custom_msg: Option<&str>,
) -> Result<bool, String> {
    let status = Command::new("git")
        .args(["status", "--porcelain"])
        .output()
        .map_err(|e| format!("Échec git status: {e}"))?;
    let changed = !status.stdout.is_empty();

    if changed {
        let add_status = Command::new("git")
            .args(["add", "."])
            .status()
            .map_err(|e| format!("git add échoué: {e}"))?;
        if !add_status.success() {
            return Err("git add échoué".to_string());
        }

        let commit_msg = custom_msg.unwrap_or(default_commit_msg);
        let commit_status = Command::new("git")
            .args(["commit", "-m", commit_msg])
            .status()
            .map_err(|e| format!("commit échoué: {e}"))?;
        if !commit_status.success() {
            return Err("commit échoué".to_string());
        }
        Ok(true)
    } else {
        Ok(false)
    }
}
