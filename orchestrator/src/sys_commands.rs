use std::process::{Child, Command};

/// Commandes système bas niveau pour la gestion des processus :
/// - Lancement de processus
/// - Kill de processus
/// - Vérification du statut (vivant/mort)

#[allow(dead_code)]
pub fn spawn_process(program: &str, args: &[&str]) -> std::io::Result<Child> {
    Command::new(program).args(args).spawn()
}

pub fn kill_process(child: &mut Child) -> std::io::Result<()> {
    child.kill()
}

/// Vérifie si un processus est toujours en vie
/// Retourne None si vivant, Some(code) si terminé
pub fn check_process_status(child: &mut Child) -> std::io::Result<Option<i32>> {
    child
        .try_wait()
        .map(|status| status.map(|s| s.code().unwrap_or(0)))
}
