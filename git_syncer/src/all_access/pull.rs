use std::process::Command;

pub fn pull_base_branch(base: &str, remote: &str) -> Result<(), String> {
    let checkout_status = Command::new("git")
        .args(["checkout", base])
        .status()
        .map_err(|e| format!("checkout base échoué: {e}"))?;
    if !checkout_status.success() {
        return Err("checkout base échoué".to_string());
    }
    let pull_status = Command::new("git")
        .args(["pull", remote, base])
        .status()
        .map_err(|e| format!("pull base échoué: {e}"))?;
    if pull_status.success() {
        Ok(())
    } else {
        Err("pull base échoué".to_string())
    }
}
