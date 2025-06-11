use crate::genome::{FileGene, GenomeConfig};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Synchronise les fichiers de structure (mod.rs, imports dans main.rs)
/// avec le génome de l'agent
pub fn sync_code_with_genome(genome: &GenomeConfig, agent_path: &Path) -> std::io::Result<bool> {
    let mut changes_made = false;

    // 1. Synchroniser les déclarations de modules dans les fichiers mod.rs
    if update_module_declarations(genome, agent_path)? {
        changes_made = true;
    }

    // 2. Synchroniser les imports et activations dans main.rs
    if update_main_file_imports(genome, agent_path)? {
        changes_made = true;
    }

    Ok(changes_made)
}

/// Met à jour les déclarations de modules dans les fichiers mod.rs
fn update_module_declarations(genome: &GenomeConfig, agent_path: &Path) -> std::io::Result<bool> {
    let mut changes_made = false;

    // Regrouper les fichiers par répertoire en conservant les chemins relatifs complets
    let mut dirs_modules: HashMap<String, Vec<&FileGene>> = HashMap::new();

    for file in &genome.files {
        let path = Path::new(&file.path);
        if let Some(parent) = path.parent() {
            let parent_str = parent.to_string_lossy().to_string();
            dirs_modules.entry(parent_str).or_default().push(file);
        }
    }

    // Pour chaque répertoire, vérifier/mettre à jour le fichier mod.rs
    for (dir, modules) in dirs_modules {
        if dir.is_empty() {
            continue; // Ignorer le répertoire racine
        }

        // Construire le chemin complet au fichier mod.rs
        // Gérer à la fois les chemins relatifs à src/ et les chemins absolus
        let mod_path = if dir.starts_with("src/") {
            agent_path.join(&dir).join("mod.rs")
        } else {
            agent_path.join("src").join(&dir).join("mod.rs")
        };

        let expected_content = generate_mod_content(&modules);

        // Vérifier si le fichier existe et a le bon contenu
        let current_content = if mod_path.exists() {
            fs::read_to_string(&mod_path)?
        } else {
            String::new()
        };

        if current_content != expected_content {
            // Créer le répertoire si nécessaire
            if let Some(parent) = mod_path.parent() {
                fs::create_dir_all(parent)?;
            }

            fs::write(&mod_path, expected_content)?;
            changes_made = true;
            println!("📝 Mise à jour du fichier mod.rs dans {}", dir);
        }
    }

    Ok(changes_made)
}

/// Génère le contenu d'un fichier mod.rs
fn generate_mod_content(modules: &[&FileGene]) -> String {
    let mut mod_content = String::new();

    for file in modules {
        if !file.active {
            continue;
        }

        let path = Path::new(&file.path);
        if let Some(file_stem) = path.file_stem() {
            let module_name = file_stem.to_string_lossy();

            // Ne pas inclure mod.rs lui-même
            if module_name != "mod" {
                mod_content.push_str(&format!("pub mod {};\n", module_name));
            }
        }
    }

    mod_content
}

/// Met à jour les imports dans main.rs
fn update_main_file_imports(genome: &GenomeConfig, agent_path: &Path) -> std::io::Result<bool> {
    let main_path = agent_path.join("src/main.rs");
    if !main_path.exists() {
        return Ok(false);
    }

    let current_content = fs::read_to_string(&main_path)?;

    // Analyser le contenu actuel pour séparer les imports, le code principal, et les appels conditionnels
    let (current_imports, main_body, _current_conditions) = parse_main_file(&current_content);

    // Générer les nouveaux imports basés sur le génome
    let mut new_imports = Vec::new();
    for file in &genome.files {
        if !file.active {
            continue;
        }

        let path = Path::new(&file.path);
        if let Some(file_stem) = path.file_stem() {
            let module_name = file_stem.to_string_lossy();
            if module_name != "main" {
                new_imports.push(format!("mod {};", module_name));
            }
        }
    }

    // Générer les nouvelles conditions d'activation basées sur le génome
    let mut new_conditions = String::new();
    for file in &genome.files {
        if !file.active {
            continue;
        }

        let path = Path::new(&file.path);
        if let Some(file_stem) = path.file_stem() {
            let module_name = file_stem.to_string_lossy();
            if module_name != "main" {
                new_conditions.push_str(&format!(
                    "    if genome.is_module_active(\"{}\") {{\n",
                    module_name
                ));
                new_conditions.push_str(&format!("        {}::run();\n", module_name));
                new_conditions.push_str("    }\n");
            }
        }
    }

    // Comparer avec le contenu actuel
    let sorted_current_imports: Vec<_> = current_imports.iter().map(|s| s.as_str()).collect();
    let sorted_new_imports: Vec<_> = new_imports.iter().map(|s| s.as_str()).collect();

    if sorted_current_imports == sorted_new_imports {
        // Pas de changement nécessaire pour les imports
        return Ok(false);
    }

    // Reconstruire le fichier main.rs avec les nouveaux imports et conditions
    let mut new_content = String::new();
    for import in &new_imports {
        new_content.push_str(import);
        new_content.push('\n');
    }
    new_content.push('\n');
    new_content.push_str(&main_body);

    // Remplacer les conditions existantes par les nouvelles conditions
    let main_fn_start = "fn main() {";
    let main_fn_replacement = format!("fn main() {{\n{}", new_conditions);

    let new_content = new_content.replace(main_fn_start, &main_fn_replacement);

    // Écrire le nouveau contenu
    fs::write(&main_path, new_content)?;

    Ok(true)
}

/// Analyse le fichier main.rs pour séparer les imports, le corps principal, et les conditions
fn parse_main_file(content: &str) -> (Vec<String>, String, String) {
    let mut imports = Vec::new();
    let mut main_body = String::new();
    let mut conditions = String::new();

    let mut in_main = false;
    let mut capture_conditions = false;

    for line in content.lines() {
        if line.trim().starts_with("mod ") && !in_main {
            imports.push(line.trim().to_string());
        } else if line.contains("fn main()") {
            in_main = true;
            main_body.push_str(line);
            main_body.push('\n');
            capture_conditions = true;
        } else if in_main && capture_conditions && line.contains("if genome.is_module_active") {
            // Capturer les conditions jusqu'à la fin de main()
            conditions.push_str(line);
            conditions.push('\n');
        } else if in_main && line.trim() == "}" && capture_conditions {
            // Fin de main()
            capture_conditions = false;
            main_body.push_str(line);
            main_body.push('\n');
        } else if !in_main {
            main_body.push_str(line);
            main_body.push('\n');
        }
    }

    (imports, main_body, conditions)
}
