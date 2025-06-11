use crate::{cargo_commands, sys_commands};
use std::path::Path;
use std::process::Child;

pub fn run_agent(agent_path: &Path) -> std::io::Result<Child> {
    let manifest_path = format!("{}/Cargo.toml", agent_path.display());
    cargo_commands::run(&manifest_path)
}

pub fn kill_agent(is_running: &mut bool, child: &mut Child) -> std::io::Result<()> {
    let result = sys_commands::kill_process(child);
    if result.is_ok() {
        *is_running = false;
    }
    result
}

pub fn reload_agent(
    agent_path: &Path,
    is_running: &mut bool,
    old_process: Option<&mut Child>,
) -> std::io::Result<Option<Child>> {
    // Kill l'ancien si existe
    if let Some(child) = old_process {
        let _ = child.kill();
        *is_running = false;
    }

    // Build
    let manifest_path = format!("{}/Cargo.toml", agent_path.display());
    if cargo_commands::build(&manifest_path)?.status.success() {
        // Run
        match cargo_commands::run(&manifest_path) {
            Ok(child) => {
                *is_running = true;
                Ok(Some(child))
            }
            Err(e) => {
                *is_running = false;
                Err(e)
            }
        }
    } else {
        *is_running = false;
        Ok(None)
    }
}

pub fn run_all_agents(agent_paths: &[std::path::PathBuf]) -> Vec<std::io::Result<Child>> {
    agent_paths.iter().map(|p| run_agent(p)).collect()
}

/// Vérifie si un agent a crashé en vérifiant son état système
pub fn check_agent_crashed(is_running: bool, child: &mut Child) -> bool {
    is_running && sys_commands::check_process_status(child).map_or(true, |status| status.is_some())
}
