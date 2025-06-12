use clap::Parser;
use colored::*;

#[derive(Parser, Debug)]
#[clap(
    version = "0.1.0",
    author = "Rémi <ton@mail.com>",
    about = "Synchronise la branche courante avec main (fetch, rebase, push, etc.)"
)]
pub struct Args {
    /// Branche de référence (par défaut "main")
    #[clap(short, long, default_value = "main")]
    pub base: String,

    /// Remote à utiliser (par défaut "origin")
    #[clap(short, long, default_value = "origin")]
    pub remote: String,

    /// Action à effectuer (rebase, merge, pull-only)
    #[clap(short, long, default_value = "rebase", value_parser = ["rebase", "merge", "pull-only"])]
    pub action: String,

    /// Utiliser force-with-lease sur le push
    #[clap(long)]
    pub force: bool,

    /// Lister les branches à synchroniser puis quitter
    #[clap(long)]
    pub list: bool,

    /// Ouvrir une PR GitHub après push (nécessite gh CLI)
    #[clap(long)]
    pub pr: bool,

    /// Loguer dans un fichier
    #[clap(long)]
    pub log: Option<String>,

    /// Mode non-interactif (désactive les prompts)
    #[clap(long)]
    pub yes: bool,

    /// Demande à l’IA de refactoriser un fichier ou dossier (ex: --refactor src/lib.rs)
    #[clap(long)]
    pub refactor: Option<String>,

    /// Demande à l’IA de réparer un fichier ou dossier (ex: --repair src/main.rs)
    #[clap(long)]
    pub repair: Option<String>,

    /// Demande à l’IA de générer du code pour un fichier ou dossier (ex: --generate src/utils.rs)
    #[clap(long)]
    pub generate: Option<String>,

    /// Démarre l’IA autonome dans un thread séparé
    #[clap(long)]
    pub start_ia: bool,

    /// Arrête l’IA autonome (si lancée en tâche de fond)
    #[clap(long)]
    pub stop_ia: bool,

    /// Affiche le status de l’IA (running/stopped, dernier état)
    #[clap(long)]
    pub status: bool,

    /// Met à jour le projet et l’IA (git pull + rebuild)
    #[clap(long)]
    pub self_update: bool,

    /// Annule la dernière action IA (undo IA, git reset --hard)
    #[clap(long)]
    pub undo_ia: bool,

    /// Lance l’IA autonome (mode expérimental)
    #[clap(long)]
    pub start_ia_autopilot: bool,
}

/// Affiche et log le message de succès final.
pub fn print_success(branch: &str, logfile: &Option<String>) {
    let final_msg = format!("✅ Branche '{}' synchronisée !", branch)
        .green()
        .bold();
    println!("{}", final_msg);
    crate::git_utils::log_message(logfile, &final_msg.to_string());
}
