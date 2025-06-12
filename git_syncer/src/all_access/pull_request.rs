use std::process::Command;

pub fn open_github_pr(base: &str, branch: &str) -> Result<(), String> {
    let status = Command::new("gh")
        .args(["pr", "create", "--fill", "--base", base, "--head", branch])
        .status()
        .map_err(|e| format!("Impossible d'ouvrir la PR avec gh CLI: {e}"))?;
    if status.success() {
        Ok(())
    } else {
        Err("Impossible d'ouvrir la PR avec gh CLI".to_string())
    }
}
