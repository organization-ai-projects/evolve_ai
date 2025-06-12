pub fn get_current_branch() -> String {
    let output = std::process::Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .expect("git rev-parse failed");
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

pub fn get_git_diff() -> String {
    let output = std::process::Command::new("git")
        .args(["diff", "--cached"])
        .output()
        .expect("git diff failed");
    String::from_utf8_lossy(&output.stdout).to_string()
}

// Ajoute ici d'autres helpers contextuels si besoin (remote, status, etc.)
