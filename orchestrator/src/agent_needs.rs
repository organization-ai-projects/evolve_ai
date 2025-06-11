use crate::agent_listing::AgentInfo;
use crate::agent_structural_code::AgentStructuralCode;

/// Applique les variations d'énergie selon les patterns de code détectés.
/// Peut être enrichi avec d'autres besoins (manger, dormir, etc.)
pub fn apply_energy_loss(agent: &mut AgentInfo, code_metrics: &AgentStructuralCode) {
    let mut energy_loss = 0;
    energy_loss += code_metrics.unwrap_count as i32 * 10;
    // Ajoute ici d'autres patterns si tu ajoutes expect_count, unsafe_count, etc.
    // energy_loss += code_metrics.expect_count as i32 * 10;
    // energy_loss += code_metrics.unsafe_count as i32 * agent.energy;

    agent.energy -= energy_loss;
}
