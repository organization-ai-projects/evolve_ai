use crate::agent_config::AgentConfig;
use crate::agent_listing::AgentInfo;
use crate::agent_selection::SelectionCriteria;
use crate::agent_structural_code::AgentStructuralCode;
use crate::pipelines::selection_life;
use std::collections::HashMap;
use std::process::Child;

pub fn process_natural_selection(
    agent: &mut AgentInfo,
    processes: &mut HashMap<String, Child>,
    total_metrics: &AgentStructuralCode,
    config: &AgentConfig,
    criteria: &SelectionCriteria,
) {
    if let Some(process) = processes.get_mut(&agent.name) {
        let survived = selection_life::process_natural_selection(
            agent,
            process,
            total_metrics,
            config,
            criteria,
        );

        // Si l'agent ne survit pas, le supprimer de la liste des processus
        if !survived {
            processes.remove(&agent.name);
        } else {
            // L'agent survit : on réinitialise son compteur de crashs
            agent.reset_crash_count();
        }
    }
}
