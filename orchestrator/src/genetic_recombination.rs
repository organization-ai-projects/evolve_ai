use crate::genome::{FileGene, GenomeConfig};
use rand::Rng;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Effectue une recombinaison génétique entre deux génomes parents
/// pour créer un nouveau génome enfant
pub fn recombine_genomes(parent1: &GenomeConfig, parent2: &GenomeConfig) -> GenomeConfig {
    let mut rng = rand::thread_rng();
    let mut child_files = Vec::new();

    // Identifier tous les fichiers présents dans au moins un des parents
    let mut all_paths: HashMap<String, bool> = HashMap::new();
    for file in &parent1.files {
        all_paths.insert(file.path.clone(), true);
    }
    for file in &parent2.files {
        all_paths.insert(file.path.clone(), true);
    }

    // Pour chaque fichier, hériter des caractéristiques d'un des parents ou faire un mélange
    for path in all_paths.keys() {
        let parent1_file = parent1.files.iter().find(|f| &f.path == path);
        let parent2_file = parent2.files.iter().find(|f| &f.path == path);

        match (parent1_file, parent2_file) {
            (Some(file1), Some(file2)) => {
                // Les deux parents ont ce fichier, mélanger les caractéristiques
                let active = if rng.gen_bool(0.5) {
                    file1.active
                } else {
                    file2.active
                };
                let mut functions = HashMap::new();

                // Mélanger les fonctions des deux parents
                let all_functions: HashMap<String, bool> = file1
                    .functions
                    .iter()
                    .chain(file2.functions.iter())
                    .map(|(k, v)| (k.clone(), *v))
                    .collect();

                for (func_name, _) in &all_functions {
                    let use_parent1 = rng.gen_bool(0.5);
                    let active = if use_parent1 {
                        file1.functions.get(func_name).copied().unwrap_or(false)
                    } else {
                        file2.functions.get(func_name).copied().unwrap_or(false)
                    };
                    functions.insert(func_name.clone(), active);
                }

                child_files.push(FileGene {
                    path: path.clone(),
                    active,
                    functions,
                });
            }
            (Some(file), None) => {
                // Seul parent1 a ce fichier
                // Utiliser clone() puisque maintenant FileGene implémente Clone
                child_files.push(file.clone());
            }
            (None, Some(file)) => {
                // Seul parent2 a ce fichier
                // Utiliser clone() puisque maintenant FileGene implémente Clone
                child_files.push(file.clone());
            }
            _ => unreachable!(),
        }
    }

    GenomeConfig { files: child_files }
}

/// Applique le génome d'un agent à ses fichiers source
/// en générant les fichiers mod.rs et en modifiant main.rs
pub fn apply_genome_to_source(genome: &GenomeConfig, agent_path: &Path) -> std::io::Result<()> {
    // 1. Générer/mettre à jour les fichiers mod.rs
    update_module_declarations(genome, agent_path)?;

    // 2. Mettre à jour le fichier main.rs
    update_main_file(genome, agent_path)?;

    Ok(())
}

/// Met à jour les déclarations de modules dans les fichiers mod.rs
fn update_module_declarations(genome: &GenomeConfig, agent_path: &Path) -> std::io::Result<()> {
    // Regrouper les fichiers par répertoire
    let mut dirs_modules: HashMap<String, Vec<FileGene>> = HashMap::new();

    for file in &genome.files {
        if !file.active {
            continue;
        }

        let path = Path::new(&file.path);
        if let Some(parent) = path.parent() {
            let parent_str = parent.to_string_lossy().to_string();
            // Maintenant que FileGene implémente Clone, on peut l'utiliser directement
            dirs_modules
                .entry(parent_str)
                .or_default()
                .push(file.clone());
        }
    }

    // Pour chaque répertoire, créer/mettre à jour le fichier mod.rs
    for (dir, modules) in dirs_modules {
        if dir.is_empty() {
            continue; // Ignorer le répertoire racine
        }

        let mod_path = agent_path.join(&dir).join("mod.rs");
        let mut mod_content = String::new();

        for module in modules {
            let module_name = Path::new(&module.path)
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy();

            // Ne pas inclure mod.rs lui-même
            if module_name != "mod" {
                mod_content.push_str(&format!("pub mod {};\n", module_name));
            }
        }

        // Créer le répertoire si nécessaire
        if let Some(parent) = mod_path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(mod_path, mod_content)?;
    }

    Ok(())
}

/// Met à jour le fichier main.rs pour activer/désactiver les modules
fn update_main_file(genome: &GenomeConfig, agent_path: &Path) -> std::io::Result<()> {
    let main_path = agent_path.join("src/main.rs");

    // Lire le contenu actuel
    let current_content = fs::read_to_string(&main_path)?;

    // Trouver la section des imports
    let mut new_content = String::new();
    let mut import_section = String::new();

    // Ajouter les imports pour les modules actifs
    for file in &genome.files {
        if !file.active {
            continue;
        }

        let path = Path::new(&file.path);
        if let Some(file_name) = path.file_stem() {
            let module_name = file_name.to_string_lossy();

            // Ne pas inclure main.rs lui-même
            if module_name != "main" {
                // Déterminer le chemin d'importation relatif
                let parent = path.parent().unwrap_or_else(|| Path::new(""));
                let import_path = if parent.to_string_lossy().is_empty() {
                    format!("mod {};", module_name)
                } else {
                    let parent_path = parent.to_string_lossy().replace("/", "::");
                    format!("mod {};", parent_path)
                };

                import_section.push_str(&format!("{}\n", import_path));
            }
        }
    }

    // Construire le contenu principal
    let mut in_fn_main = false;
    let mut in_imports = true;

    for line in current_content.lines() {
        if line.trim().starts_with("fn main()") {
            in_fn_main = true;
            in_imports = false;
            new_content.push_str(line);
            new_content.push('\n');
            continue;
        }

        if in_imports {
            if line.trim().starts_with("mod ") {
                // Ignorer les imports existants, on les remplacera
                continue;
            }
            if !line.trim().is_empty() && !line.trim().starts_with("//") {
                in_imports = false;
            }
        }

        if !in_imports && !in_fn_main {
            new_content.push_str(line);
            new_content.push('\n');
        }

        if in_fn_main {
            if line.contains("}") && !line.contains("{") {
                in_fn_main = false;
                new_content.push_str(line);
                new_content.push('\n');
            } else if line.contains("if genome.is_module_active") {
                // Remplacer cette partie par notre logique basée sur le génome
                continue;
            } else {
                new_content.push_str(line);
                new_content.push('\n');
            }
        }
    }

    // Insérer les imports au début
    let final_content = import_section + &new_content;

    // Générer la section du code activant les modules en fonction du génome
    let mut activation_code = String::new();
    activation_code.push_str("    let genome = genome::Genome::random();\n");
    activation_code.push_str("    println!(\"Agent genome: {:?}\", genome);\n\n");

    for file in &genome.files {
        if !file.active {
            continue;
        }

        let path = Path::new(&file.path);
        if let Some(file_name) = path.file_stem() {
            let module_name = file_name.to_string_lossy();
            if module_name != "main" {
                activation_code.push_str(&format!(
                    "    if genome.is_module_active(\"{}\") {{\n",
                    module_name
                ));
                activation_code.push_str(&format!("        {}::run();\n", module_name));
                activation_code.push_str("    }\n");
            }
        }
    }

    // Insérer le code d'activation après la déclaration de fn main() {
    let final_content = final_content.replace(
        "fn main() {\n",
        &format!("fn main() {{\n{}", activation_code),
    );

    fs::write(main_path, final_content)?;

    Ok(())
}
