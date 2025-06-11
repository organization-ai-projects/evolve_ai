use crate::agent_file_hashes::AgentFileHashes;
use crate::agent_listing::AgentInfo;
use crate::agent_sanitizer;
use crate::agent_validator;
use crate::scan_agents::AgentScanResult;
use std::path::PathBuf;

pub fn build_agent_info(
    agent_id: String,
    short_uuid: String,
    agent_dir: PathBuf,
    scan_result: &AgentScanResult,
) -> AgentInfo {
    AgentInfo {
        id: agent_id,
        name: short_uuid,
        path: agent_dir.clone(),
        active: true,
        code_hash: scan_result.code_hash.clone(),
        file_hashes: AgentFileHashes {
            code_hash: scan_result.code_hash.clone(),
            file_hashes: scan_result.file_hashes.clone(),
        },
        file_metrics: scan_result.file_metrics.clone(),
        is_safe: agent_sanitizer::is_code_safe(
            &scan_result
                .files
                .iter()
                .map(|(p, c)| (p.clone(), c.clone()))
                .collect::<Vec<_>>(),
            &agent_dir.to_string_lossy(),
        ),
        is_valid: agent_validator::is_code_valid(&agent_dir),
        is_running: false,
        last_modified: std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        energy: 1000,
        last_crash: None,
        crash_count: 0,
    }
}
