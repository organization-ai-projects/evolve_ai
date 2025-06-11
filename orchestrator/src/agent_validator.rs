use crate::cargo_commands;
use std::path::Path;

/// Vérifie si le code de l'agent compile correctement (check cargo)
/// C'est la seule méthode publique à utiliser pour la validité.
pub fn is_code_valid(agent_path: &Path) -> bool {
    let manifest_path = format!("{}/Cargo.toml", agent_path.display());
    match cargo_commands::check(&manifest_path) {
        Ok(output) => output.status.success(),
        Err(_) => false,
    }
}

// --- Helpers privés éventuels (scan_rust_files, etc.) ---
