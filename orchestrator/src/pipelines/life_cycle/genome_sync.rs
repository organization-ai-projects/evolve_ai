use crate::agent_listing::AgentInfo;
use crate::genome::GenomeConfig;
use crate::genome_sync;
use crate::project_paths::ProjectPaths;

pub fn sync_agent_with_genome(agent: &mut AgentInfo, paths: &ProjectPaths) {
    let genome_path = paths.agent_genome_path(&agent.name);
    if let Ok(genome_bytes) = std::fs::read(&genome_path) {
        if let Ok(genome) = bincode::deserialize::<GenomeConfig>(&genome_bytes) {
            if let Ok(true) = genome_sync::sync_code_with_genome(&genome, &agent.path) {
                println!(
                    "📐 Synchronisation du code avec le génome pour {}",
                    agent.name
                );
                agent.file_hashes.code_hash = "".to_string();
            }
        }
    }
}
