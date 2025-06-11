//mutation pour ajouter de nouveaux modules internes dans le fichier main.rs
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileGene {
    // Renommé de FileConfig à FileGene
    pub path: String, // Ajout du commentaire explicatif
    pub active: bool,
    pub functions: HashMap<String, bool>, // Ajout du commentaire explicatif
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GenomeConfig {
    // Renommé de AgentConfig à GenomeConfig
    pub files: Vec<FileGene>,
}

pub struct ModuleManager {
    config_file: PathBuf,
    config: GenomeConfig, // Mis à jour pour utiliser le nouveau nom
}

impl ModuleManager {
    pub fn new(config_file: PathBuf) -> Self {
        Self {
            config_file,
            config: GenomeConfig { files: Vec::new() },
        }
    }

    pub fn mutate(&mut self) -> std::io::Result<()> {
        let mut rng = rand::thread_rng();

        if let Some(file) = self.config.files.iter_mut().choose(&mut rng) {
            // 50% chance de muter le fichier ou une fonction
            if rng.gen_bool(0.5) {
                file.active = !file.active;
            } else if let Some((_, active)) = file.functions.iter_mut().choose(&mut rng) {
                *active = !*active;
            }
        }

        self.save_state()
    }

    pub fn save_state(&self) -> std::io::Result<()> {
        let content = ron::ser::to_string_pretty(
            &self.config,
            ron::ser::PrettyConfig::default().with_enumerate_arrays(true),
        )
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        fs::write(&self.config_file, content)
    }

    pub fn load_state(&mut self) -> std::io::Result<()> {
        if self.config_file.exists() {
            let content = fs::read_to_string(&self.config_file)?;
            self.config = ron::from_str(&content)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        }
        Ok(())
    }
}
