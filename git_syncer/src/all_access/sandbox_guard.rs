use std::path::Path;

/// Vérifie qu'un chemin est bien dans sandbox/
pub fn is_in_sandbox(path: &str) -> bool {
    let sandbox = Path::new("git_syncer/sandbox");
    let abs = std::fs::canonicalize(path).unwrap_or_else(|_| Path::new(path).to_path_buf());
    abs.starts_with(sandbox)
}

/// Refuse toute commande dangereuse ou hors sandbox
pub fn is_command_safe(cmd: &str, args: &[&str], cwd: &str) -> bool {
    let forbidden = ["rm", "del", "mv", "cp", "shutdown", "reboot"];
    if forbidden.iter().any(|&f| cmd.contains(f)) {
        return false;
    }
    if args.iter().any(|a| a.contains("unsafe")) {
        return false;
    }
    is_in_sandbox(cwd)
}
