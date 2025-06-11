use std::process::{Child, Command, Output};

/// Commandes cargo génériques :
/// - check: vérifie la compilation d'un projet Rust
/// - build: compile un projet Rust
/// - run: lance un projet Rust via cargo

pub fn check(manifest_path: &str) -> std::io::Result<Output> {
    Command::new("cargo")
        .args(["check", "--manifest-path", manifest_path])
        .output()
}

pub fn build(manifest_path: &str) -> std::io::Result<Output> {
    Command::new("cargo")
        .args(["build", "--manifest-path", manifest_path])
        .output()
}

pub fn run(manifest_path: &str) -> std::io::Result<Child> {
    Command::new("cargo")
        .args(["run", "--manifest-path", manifest_path])
        .spawn()
}
