// src/main.rs

mod ai;
mod all_access;
mod cli;
mod human;
mod main_content;

use clap::Parser;
use cli::Args;
use main_content::ai_main::cli_main::handle_ai_cli;
use main_content::all::main_all::route_generate_code;
use main_content::human_main::cli_main::handle_cli_mode;

fn main() {
    let args = Args::parse();

    // Si un mode IA est déclenché, on ne fait rien d’autre
    if handle_ai_cli(&args) {
        return;
    }

    // Exemple d'appel du routeur pour une demande humain -> IA
    // (à placer là où tu veux permettre l'interaction)
    // let res = route_generate_code("génère une fonction add dans math.rs", BridgeDirection::HumanToIa);
    // println!("Résultat IA :\n{}", res);

    // Sinon, on exécute le workflow CLI classique
    handle_cli_mode(&args);
}
