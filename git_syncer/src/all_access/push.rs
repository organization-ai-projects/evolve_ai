use std::process::Command;

pub fn push_branch(remote: &str, branch: &str, force: bool) -> Result<(), String> {
    let push_args = if force {
        vec!["push", "--force-with-lease", remote, branch]
    } else {
        vec!["push", remote, branch]
    };
    let status = Command::new("git")
        .args(push_args)
        .status()
        .map_err(|e| format!("Erreur d'exécution git push: {e}"))?;
    if status.success() {
        Ok(())
    } else {
        Err("push échoué".to_string())
    }
}
