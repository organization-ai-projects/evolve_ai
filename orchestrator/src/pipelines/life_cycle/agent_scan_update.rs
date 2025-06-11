use crate::agent_listing::AgentInfo;
use crate::agent_structural_code::AgentStructuralCode;
use crate::scan_agents::RustScanner;
use std::collections::HashMap;

pub struct ScanUpdateResult {
    pub changed_files: Vec<String>,
    pub scan_files: Vec<(String, String)>,
    pub scan_result_code_hash: String,
}

pub fn scan_and_update_agent(
    agent: &mut AgentInfo,
    scanner: &RustScanner,
    agent_file_hashes: &mut HashMap<String, HashMap<String, String>>,
    agent_file_metrics: &mut HashMap<String, HashMap<String, AgentStructuralCode>>,
) -> Option<ScanUpdateResult> {
    let prev_hashes = agent.file_hashes.file_hashes.clone();
    let scan_result = match scanner.scan_agent(&agent.path, false) {
        Ok(res) => res,
        Err(e) => {
            eprintln!("Erreur lors du scan de l'agent {}: {}", agent.name, e);
            return None;
        }
    };

    let changed_files: Vec<String> = scan_result
        .file_hashes
        .iter()
        .filter_map(|(file, hash)| {
            if prev_hashes.get(file) != Some(hash) {
                Some(file.clone())
            } else {
                None
            }
        })
        .collect();

    agent_file_hashes.insert(agent.name.clone(), scan_result.file_hashes.clone());
    agent_file_metrics.insert(agent.name.clone(), scan_result.file_metrics.clone());

    agent.file_hashes.file_hashes = scan_result.file_hashes.clone();
    agent.file_metrics = scan_result.file_metrics.clone();

    let scan_files: Vec<(String, String)> = scan_result
        .files
        .iter()
        .map(|(p, c)| (p.clone(), c.clone()))
        .collect();

    Some(ScanUpdateResult {
        changed_files,
        scan_files,
        scan_result_code_hash: scan_result.code_hash.clone(),
    })
}
