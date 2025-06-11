use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GeneConfig {
    pub active: bool,
    pub mutation_rate: f32,
    pub parameters: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AgentGenes {
    pub id: String,
    pub modules: HashMap<String, GeneConfig>,
    pub generation: u32,
}

impl AgentGenes {
    pub fn new(id: String) -> Self {
        Self {
            id,
            modules: [
                ("neural", true),
                ("memory_graph", true),
                ("symbolic", true),
                ("curiosity", true),
                ("fsm", true),
                ("rl", true),
                ("hormones", true),
                ("communication", true),
                ("selfmod", true),
                ("meta", true),
            ]
            .iter()
            .map(|(name, active)| {
                (
                    name.to_string(),
                    GeneConfig {
                        active: *active,
                        mutation_rate: 0.1,
                        parameters: HashMap::new(),
                    },
                )
            })
            .collect(),
            generation: 0,
        }
    }

    pub fn new_from_modules(modules: &[String]) -> Self {
        let modules_map = modules
            .iter()
            .map(|name| {
                (
                    name.clone(),
                    GeneConfig {
                        active: false, // Inactif par défaut
                        mutation_rate: 0.1,
                        parameters: HashMap::new(),
                    },
                )
            })
            .collect();

        Self {
            id: uuid::Uuid::new_v4().to_string(),
            modules: modules_map,
            generation: 0,
        }
    }

    pub fn save_ron(&self, path: &Path) -> std::io::Result<()> {
        let content = ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::default())
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        fs::write(path, content)
    }

    pub fn load_ron(path: &Path) -> std::io::Result<Self> {
        let content = fs::read_to_string(path)?;
        ron::from_str(&content).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
    }

    pub fn save_binary(&self, path: &Path) -> std::io::Result<()> {
        let encoded: Vec<u8> = bincode::serialize(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        fs::write(path, encoded)
    }

    pub fn load_binary(path: &Path) -> std::io::Result<Self> {
        let data = fs::read(path)?;
        bincode::deserialize(&data).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
    }
}
