use std::path::{Path, PathBuf};

// Définition des constantes pour les chemins
pub const WORKSPACE_DIR: &str = "evolve_ai"; // Racine du workspace evolve_ai/
pub const ORCHESTRATOR_DIR: &str = "orchestrator"; // Sous-dossier orchestrator
pub const AGENTS_DIR: &str = "agents"; // Sous-dossier agents
pub const TEMPLATE_DIR: &str = "agent_template"; // Sous-dossier template des agents

pub struct ProjectPaths {
    pub workspace_dir: PathBuf,
    pub orchestrator_dir: PathBuf,
    pub agents_dir: PathBuf,
    pub template_dir: PathBuf,
}

impl ProjectPaths {
    pub fn new() -> Self {
        let workspace_path = Path::new(WORKSPACE_DIR).to_path_buf();
        Self {
            workspace_dir: workspace_path.clone(),
            orchestrator_dir: workspace_path.join(ORCHESTRATOR_DIR),
            agents_dir: workspace_path.join(AGENTS_DIR),
            template_dir: workspace_path.join(TEMPLATE_DIR),
        }
    }

    /// Helper pour obtenir le chemin du fichier listing_agents.bin
    pub fn agent_listing_path(&self) -> PathBuf {
        self.agents_dir.join("listing_agents.bin")
    }

    /// Helper pour obtenir le chemin du fichier genome.bin d'un agent
    pub fn agent_genome_path(&self, agent_name: &str) -> PathBuf {
        self.agents_dir.join(agent_name).join("genome.bin")
    }
}
