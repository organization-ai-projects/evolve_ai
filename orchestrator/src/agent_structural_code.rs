use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AgentStructuralCode {
    pub unwrap_count: u32,
    // Ajoute d'autres métriques ici si besoin
}
