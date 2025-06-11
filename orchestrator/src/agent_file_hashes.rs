use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AgentFileHashes {
    pub code_hash: String,
    pub file_hashes: HashMap<String, String>,
}
