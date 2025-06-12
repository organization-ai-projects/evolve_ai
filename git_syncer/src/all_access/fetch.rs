use std::process::Command;

pub fn fetch_remote(remote: &str) -> Result<(), String> {
    let status = Command::new("git")
        .args(["fetch", remote])
        .status()
        .map_err(|e| format!("fetch échoué: {e}"))?;
    if status.success() {
        Ok(())
    } else {
        Err("fetch échoué".to_string())
    }
}
