use crate::all_access::branch::{
    checkout_branch, current_branch, ensure_tracking_branch, list_branches,
};
use crate::all_access::commit::auto_add_and_commit;
use crate::all_access::fetch::fetch_remote;
use crate::all_access::git_utils::{log_message, prompt_step};
use crate::all_access::pull::pull_base_branch;
use crate::all_access::pull_request::open_github_pr;
use crate::all_access::push::push_branch;
use crate::all_access::sync_action::sync_action;
use crate::cli::print_success;
use crate::cli::Args;
use crate::main_content::all::ia_bridge::{generate_code_from_prompt, BridgeDirection};
use chrono::Utc;
use colored::*;
use dialoguer::{Confirm, Input};

pub fn handle_cli_mode(args: &Args) {
    // Lister les branches si demandé
    if args.list {
        match list_branches() {
            Ok(branches) => {
                println!("{}", "🌿 Branches locales :".bold().cyan());
                for line in branches {
                    println!("  {}", line.green());
                }
            }
            Err(e) => eprintln!("{}", format!("❌ {e}").red()),
        }
        return;
    }

    // --- Commandes IA explicites ---
    if let Some(path) = &args.refactor {
        println!("🧠 Demande IA : refactoriser {path}");
        // Appelle ici la fonction IA de refactorisation (à implémenter dans ai/coding.rs)
        // ex: ai::coding::refactor_file(path);
        return;
    }
    if let Some(path) = &args.repair {
        println!("🧠 Demande IA : réparer {path}");
        // Appelle ici la fonction IA de réparation (à implémenter dans ai/coding.rs)
        // ex: ai::coding::repair_file(path);
        return;
    }
    if let Some(path) = &args.generate {
        println!("🧠 Demande IA : générer du code pour {path}");
        // Appelle ici la fonction IA de génération (à implémenter dans ai/coding.rs)
        // ex: ai::coding::generate_code_for(path);
        return;
    }

    // --- Commande IA codante via prompt ---
    if args.generate.is_none() {
        let want_code: bool = Confirm::new()
            .with_prompt("Voulez-vous demander à l'IA de générer du code ?")
            .default(false)
            .interact()
            .unwrap();
        if want_code {
            let demande: String = Input::new()
                .with_prompt("Décrivez ce que vous voulez que l'IA code (ex: 'dans utils/math.rs')")
                .interact_text()
                .unwrap();
            // Appelle le bridge IA avec direction explicite
            let res = generate_code_from_prompt(&demande, BridgeDirection::HumanToIa);
            println!("Résultat IA :\n{}", res);
            return;
        }
    }

    // --- Workflow CLI classique ---
    let logfile = &args.log;

    // 1. Branche courante
    let branch = match current_branch() {
        Ok(b) => {
            let msg = format!("📍 Branche courante : {}", b).bold();
            println!("{}", msg);
            log_message(logfile, &msg.to_string());
            b
        }
        Err(e) => {
            eprintln!("{}", format!("❌ {e}").red());
            log_message(logfile, &format!("❌ {e}"));
            std::process::exit(1);
        }
    };

    // 2. Fetch remote
    let fetch_msg = format!("🔄 git fetch {}", args.remote).cyan();
    println!("{}", fetch_msg);
    log_message(logfile, &fetch_msg.to_string());
    if prompt_step(args.yes, "Continuer avec fetch ?") {
        if let Err(e) = fetch_remote(&args.remote) {
            eprintln!("{}", format!("❌ {e}").red());
            log_message(logfile, &format!("❌ {e}"));
            std::process::exit(1);
        }
    } else {
        println!("{}", "⏭️  Étape fetch ignorée.".yellow());
    }

    // 3. Pull la base
    let pull_msg = format!("📥 Mise à jour de la branche de base '{}'", args.base).cyan();
    println!("{}", pull_msg);
    log_message(logfile, &pull_msg.to_string());
    if prompt_step(args.yes, "Continuer avec pull de la base ?") {
        if let Err(e) = pull_base_branch(&args.base, &args.remote) {
            eprintln!("{}", format!("❌ {e}").red());
            log_message(logfile, &format!("❌ {e}"));
            std::process::exit(1);
        }
    } else {
        println!("{}", "⏭️  Étape pull ignorée.".yellow());
    }

    // 4. Retour branche courante
    let retour_msg = format!("↩️  Retour sur {}", branch).cyan();
    println!("{}", retour_msg);
    log_message(logfile, &retour_msg.to_string());
    if prompt_step(args.yes, "Continuer avec checkout branche courante ?") {
        if let Err(e) = checkout_branch(&branch) {
            eprintln!("{}", format!("❌ {e}").red());
            log_message(logfile, &format!("❌ {e}"));
            std::process::exit(1);
        }
    } else {
        println!("{}", "⏭️  Étape checkout ignorée.".yellow());
    }

    // 4.bis. Auto-add & auto-commit si besoin
    let default_commit_msg = format!("chore: auto-commit from git-syncer ({})", Utc::now());
    let commit_msg = if prompt_step(args.yes, "Entrer un message de commit personnalisé ?") {
        Some(
            Input::new()
                .with_prompt("Commit message")
                .default(default_commit_msg.clone())
                .interact_text()
                .unwrap(),
        )
    } else {
        None
    };
    match auto_add_and_commit(&default_commit_msg, commit_msg.as_deref()) {
        Ok(true) => println!("{}", "✅ Commit effectué.".green()),
        Ok(false) => println!("{}", "ℹ️  Aucun changement à ajouter/commiter.".yellow()),
        Err(e) => {
            eprintln!("{}", format!("❌ {e}").red());
            log_message(logfile, &format!("❌ {e}"));
            std::process::exit(1);
        }
    }

    // 5. Action principale (rebase, merge, pull-only)
    let action_msg = format!(
        "🛠️  {} de '{}' sur '{}'",
        args.action.to_uppercase(),
        branch,
        args.base
    )
    .cyan();
    println!("{}", action_msg);
    log_message(logfile, &action_msg.to_string());
    if prompt_step(args.yes, &format!("Continuer avec {} ?", args.action)) {
        if let Err(e) = sync_action(&args.action, &branch, &args.base, &args.remote) {
            eprintln!("{}", format!("❌ {e}").red());
            log_message(logfile, &format!("❌ {e}"));
            std::process::exit(1);
        }
    } else {
        println!("{}", "⏭️  Étape action ignorée.".yellow());
    }

    // 6. Set-upstream si besoin (test push dry-run, si erreur: set upstream)
    match ensure_tracking_branch(&args.remote, &branch) {
        Ok(true) => println!("{}", "🔗 Tracking branch configuré.".green()),
        Ok(false) => {} // rien à faire
        Err(e) => {
            eprintln!("{}", format!("❌ {e}").red());
            log_message(logfile, &format!("❌ {e}"));
            std::process::exit(1);
        }
    }

    // 7. Push final
    let push_msg = format!("🚀 Push sur remote {} {}", args.remote, branch).cyan();
    println!("{}", push_msg);
    log_message(logfile, &push_msg.to_string());
    if prompt_step(args.yes, "Continuer avec le push ?") {
        match push_branch(&args.remote, &branch, args.force) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("{}", format!("❌ {e}").red());
                log_message(logfile, &format!("❌ {e}"));
                std::process::exit(1);
            }
        }
    } else {
        println!("{}", "⏭️  Étape push ignorée.".yellow());
    }

    // 8. Ouvrir une PR GitHub si demandé
    if args.pr {
        let pr_msg = "🔗 Ouverture de la Pull Request sur GitHub…".cyan();
        println!("{}", pr_msg);
        log_message(logfile, &pr_msg.to_string());
        if prompt_step(args.yes, "Ouvrir une PR GitHub ?") {
            match open_github_pr(&args.base, &branch) {
                Ok(_) => println!("{}", "✅ PR ouverte avec succès !".green()),
                Err(e) => eprintln!("{}", format!("❌ {e}").red()),
            }
        }
    }

    print_success(&branch, logfile);
}
