use crate::agent_config::AgentConfig;
use crate::agent_listing::AgentInfo;
use crate::agent_needs::apply_energy_loss;
use crate::agent_selection::{should_survive_agent, SelectionCriteria};
use crate::agent_structural_code::AgentStructuralCode;
use crate::manage_agents_commands;
use crate::monitoring::resource_monitor;
use crate::notifications::notifier;
use std::process::Child;

/// Gère la sélection naturelle d'un agent en exécutant une série de vérifications
/// par ordre de priorité. Si l'agent échoue à l'une de ces vérifications, il est
/// terminé avec une notification appropriée.
///
/// # Arguments
/// * `agent` - L'agent à évaluer
/// * `process` - Le processus de l'agent en cours d'exécution
/// * `total_metrics` - Métriques de code analysées pour l'agent
/// * `config` - Configuration globale (limites de ressources, etc.)
/// * `criteria` - Critères de sélection spécifiques
///
/// # Retourne
/// `true` si l'agent a survécu à toutes les vérifications, `false` sinon
pub fn process_natural_selection(
    agent: &mut AgentInfo,
    process: &mut Child,
    total_metrics: &AgentStructuralCode,
    config: &AgentConfig,
    criteria: &SelectionCriteria,
) -> bool {
    // --------------------------------
    // 1. DÉTECTION DE CRASH
    // --------------------------------
    // Vérifie d'abord si l'agent a crashé trop souvent
    // Si c'est le cas, inutile de continuer les autres vérifications
    if manage_agents_commands::check_agent_crashed(agent.is_running, process) {
        agent.increment_crash(); // Utilise la méthode de AgentInfo

        // Si l'agent a dépassé le nombre maximal de tentatives, on le désactive
        if agent.crash_count > config.max_retries {
            agent.active = false;
            notifier::notify_disabled(agent, "Too many crashes".into());
            return false;
        } else {
            // Sinon on notifie le crash mais on laisse une chance
            notifier::notify_crashed(agent, "Agent crash détecté".into());
        }
    }

    // --------------------------------
    // 2. VÉRIFICATION DES RESSOURCES
    // --------------------------------
    // Termine l'agent s'il consomme trop de ressources système
    let over_limit = resource_monitor::check_resource_usage(
        process,
        config.memory_limit_mb,
        config.cpu_limit_percent,
    );

    if over_limit {
        notifier::notify_resource_limit(agent, "memory/cpu".into(), config.memory_limit_mb);
        notifier::notify_killed(agent, "Agent tué pour dépassement de ressources".into());
        if let Err(e) = manage_agents_commands::kill_agent(&mut agent.is_running, process) {
            eprintln!("Failed to kill agent {}: {}", agent.name, e);
        }
        // Ne supprime plus l'agent de processes ici
        return false;
    }

    // --------------------------------
    // 3. GESTION DE L'ÉNERGIE
    // --------------------------------
    // Applique la perte d'énergie basée sur les patterns de code
    apply_energy_loss(agent, total_metrics);

    // Vérifie si l'agent a encore assez d'énergie pour fonctionner
    if agent.energy <= 0 {
        agent.active = false;
        notifier::notify_disabled(agent, "Agent épuisé (énergie nulle)".into());
        return false;
    }

    // --------------------------------
    // 4. RÈGLES DE SÉLECTION NATURELLE
    // --------------------------------
    // Applique un ensemble de règles complexes pour déterminer si l'agent
    // doit survivre (sécurité, validité, qualité du code, etc.)
    if !should_survive_agent(
        agent.is_safe,
        agent.is_valid,
        0,
        0,
        config.memory_limit_mb,
        config.cpu_limit_percent,
        total_metrics,
        criteria,
        agent.energy,
    ) {
        notifier::notify_resource_limit(
            agent,
            "selection_naturelle".into(),
            config.memory_limit_mb,
        );
        notifier::notify_killed(agent, "Agent tué par sélection naturelle".into());
        if let Err(e) = manage_agents_commands::kill_agent(&mut agent.is_running, process) {
            eprintln!("Failed to kill agent {}: {}", agent.name, e);
        }
        // Ne supprime plus l'agent de processes ici
        return false;
    }

    // L'agent a survécu à toutes les vérifications
    true
}
