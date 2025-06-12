use colored::*;
use dialoguer::{theme::ColorfulTheme, Confirm};
use std::process::{exit, Command};

pub fn run_or_exit(cmd: &mut Command, msg: &str) {
    let status = cmd.status().expect("Erreur d'exécution");
    if !status.success() {
        eprintln!("{}", format!("❌ {msg}").red());
        exit(1);
    }
}

pub fn log_message(logfile: &Option<String>, msg: &str) {
    if let Some(path) = logfile {
        use std::fs::OpenOptions;
        use std::io::Write;
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .unwrap();
        writeln!(file, "{msg}").ok();
    }
}

pub fn prompt_step(yes: bool, msg: &str) -> bool {
    if yes {
        true
    } else {
        Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt(msg)
            .default(true)
            .interact()
            .unwrap()
    }
}

pub fn log_error(logfile: &Option<String>, msg: &str) {
    eprintln!("{}", format!("❌ {msg}").red());
    log_message(logfile, &format!("❌ {msg}"));
}
