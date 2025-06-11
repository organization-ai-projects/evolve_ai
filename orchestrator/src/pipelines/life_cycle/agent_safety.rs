use crate::agent_code_analysis::analyze_structural_code;
use crate::agent_listing::AgentInfo;
use crate::agent_sanitizer;
use crate::notifications::notifier;

pub fn check_and_handle_agent_safety_and_metrics(
    agent: &mut AgentInfo,
    scan_files: &Vec<(String, String)>,
    changed_files: &[String],
) -> bool {
    agent.is_safe = agent_sanitizer::is_code_safe(scan_files, &agent.path.to_string_lossy());
    if !agent.is_safe {
        agent.active = false;
        agent.is_valid = false;
        notifier::notify_disabled(agent, "Agent non sûr après nettoyage".into());
        return false;
    }

    // Met à jour le cache des métriques uniquement pour les fichiers modifiés
    let mut metrics_cache = agent.file_metrics.clone();
    for file in changed_files {
        let file_path = agent.path.join(file);
        let metrics = analyze_structural_code(&file_path);
        metrics_cache.insert(file.clone(), metrics);
    }
    agent.file_metrics = metrics_cache;

    true
}
