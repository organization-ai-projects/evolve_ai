use crate::genome::{FileGene, GenomeConfig};
use std::collections::HashMap;

pub fn generate_initial_genome(
    scan_files: &[(String, String)],
    rng: &mut impl rand::Rng,
) -> GenomeConfig {
    let mutation_exists = scan_files
        .iter()
        .any(|(path, _)| path.contains("modules.rs") || path.contains("genome/mutate"));

    let mut files = Vec::new();
    for (path, _) in scan_files {
        let is_main = path.contains("main.rs");
        let is_mutation = path.contains("modules.rs") || path.contains("genome/mutate");
        let active = is_main || is_mutation || rng.gen_bool(0.7);

        files.push(FileGene {
            path: path.clone(),
            active,
            functions: HashMap::new(),
        });
    }

    // S'assurer qu'au moins un module de mutation est actif
    if mutation_exists
        && !files.iter().any(|f| {
            (f.path.contains("modules.rs") || f.path.contains("genome/mutate")) && f.active
        })
    {
        if let Some(file) = files
            .iter_mut()
            .find(|f| f.path.contains("modules.rs") || f.path.contains("genome/mutate"))
        {
            file.active = true;
            println!("🧬 Activation forcée du module de mutation: {}", file.path);
        }
    }

    GenomeConfig { files }
}
