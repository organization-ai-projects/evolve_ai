use std::process::Command;

pub fn current_branch() -> Result<String, String> {
    let output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .map_err(|e| format!("Échec git rev-parse: {e}"))?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Err("Impossible de récupérer la branche courante".to_string())
    }
}

pub fn list_branches() -> Result<Vec<String>, String> {
    let output = Command::new("git")
        .args(["branch", "--format", "%(refname:short)"])
        .output()
        .map_err(|e| format!("Échec git branch: {e}"))?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(|s| s.to_string())
            .collect())
    } else {
        Err("Impossible de lister les branches".to_string())
    }
}

pub fn checkout_branch(branch: &str) -> Result<(), String> {
    let status = Command::new("git")
        .args(["checkout", branch])
        .status()
        .map_err(|e| format!("Échec git checkout: {e}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("checkout branche {branch} échoué"))
    }
}

pub fn ensure_tracking_branch(remote: &str, branch: &str) -> Result<bool, String> {
    let output = Command::new("git")
        .args(["push", "--dry-run", remote, branch])
        .output()
        .map_err(|e| format!("Échec dry-run push: {e}"))?;
    let need_set_upstream = !output.status.success()
        && String::from_utf8_lossy(&output.stderr).contains("set-upstream");
    if need_set_upstream {
        let status = Command::new("git")
            .args(["push", "--set-upstream", remote, branch])
            .status()
            .map_err(|e| format!("Échec set-upstream: {e}"))?;
        if status.success() {
            Ok(true)
        } else {
            Err("set-upstream échoué".to_string())
        }
    } else {
        Ok(false)
    }
}
