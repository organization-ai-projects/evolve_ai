use crate::agent_structural_code::AgentStructuralCode;
/// Critères de sélection naturelle pour un agent
#[derive(Debug, Clone)]
pub struct SelectionCriteria {
    pub penalty_if_not_safe: u32,
    pub penalty_if_not_valid: u32,
    pub penalty_if_resource_exceeded: u32,
    pub penalty_per_unwrap: u32, // Ajout
    pub max_score: u32,
}

impl Default for SelectionCriteria {
    fn default() -> Self {
        Self {
            penalty_if_not_safe: 1000,
            penalty_if_not_valid: 1000,
            penalty_if_resource_exceeded: 1000,
            penalty_per_unwrap: 10, // Exemple
            max_score: 1000,
        }
    }
}

/// Décide si l'agent doit survivre selon toutes les règles de sélection naturelle.
/// Le pipeline collecte les métriques et les passe ici.
pub fn should_survive_agent(
    is_safe: bool,
    is_valid: bool,
    memory_mb: u64,
    cpu_percent: u8,
    memory_limit: u64,
    cpu_limit: u8,
    code_metrics: &AgentStructuralCode,
    criteria: &SelectionCriteria,
    energy: i32, // Ajout
) -> bool {
    if energy <= 0 {
        return false;
    }
    let mut score = 0;

    if !is_safe {
        score += criteria.penalty_if_not_safe;
    }
    if !is_valid {
        score += criteria.penalty_if_not_valid;
    }
    if memory_mb > memory_limit || cpu_percent > cpu_limit {
        score += criteria.penalty_if_resource_exceeded;
    }
    score += code_metrics.unwrap_count * criteria.penalty_per_unwrap;

    score <= criteria.max_score
}
