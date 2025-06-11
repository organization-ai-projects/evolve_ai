use crate::agent_file_hashes::AgentFileHashes;
use crate::agent_structural_code::AgentStructuralCode;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

/// Structure de données pour suivre l'état d'un agent
/// - name: Nom du dossier de l'agent (UUID v7 raccourci)
/// - path: chemin vers le dossier de l'agent
/// - active: si l'agent doit être exécuté
/// - code_hash: empreinte du code pour détecter les modifications
/// - is_safe: vérifié et ne contient pas de code dangereux
/// - is_valid: compile correctement
/// - is_running: actuellement en cours d'exécution
/// - last_modified: timestamp dernière modification
/// Structure de données pour suivre l'état d'un agent
/// Structure de données pour suivre l'état d'un agent
#[derive(Debug, Serialize, Deserialize)]
pub struct AgentInfo {
    // Identifiant unique (UUID v7 complet)
    pub id: String,

    // Informations d'identité

    // Nom du dossier de l'agent (UUID v7 raccourci)
    pub name: String,
    pub path: PathBuf,

    // État opérationnel
    pub active: bool,
    pub is_running: bool,

    // Qualité et sécurité du code
    pub code_hash: String,
    pub is_safe: bool,
    pub is_valid: bool,
    pub last_modified: u64,

    // Données pour la sélection naturelle
    pub energy: i32,

    // Métriques et hashes
    pub file_hashes: AgentFileHashes,
    pub file_metrics: std::collections::HashMap<String, AgentStructuralCode>,

    // État des crashs
    pub last_crash: Option<u64>, // Timestamp UNIX
    pub crash_count: u32,
}

/// Collection d'agents sauvegardée dans listing_agents.bin
#[derive(Debug, Serialize, Deserialize)]
pub struct AgentsListing {
    pub agents: Vec<AgentInfo>,
}

impl AgentInfo {
    // Méthodes pour gérer les crashs
    pub fn increment_crash(&mut self) {
        self.crash_count += 1;
        self.last_crash = Some(
            std::time::SystemTime::now()
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        );
    }

    pub fn reset_crash_count(&mut self) {
        self.crash_count = 0;
        self.last_crash = None;
    }

    pub fn has_recent_crash(&self, backoff_delay: u64) -> bool {
        if let Some(last_crash) = self.last_crash {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            now < last_crash + backoff_delay
        } else {
            false
        }
    }
}

/// Génère un identifiant UUID v7 pour un agent
pub fn generate_agent_id() -> String {
    Uuid::now_v7().to_string() // Utilise Uuid::now_v7 pour simplifier
}

/// Génère une version raccourcie de l'UUID v7 pour le dossier
pub fn generate_short_uuid(uuid: &str) -> String {
    uuid.chars().take(8).collect()
}
