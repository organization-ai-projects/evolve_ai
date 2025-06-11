use serde::{Deserialize, Serialize};
use std::{path::PathBuf, time::Duration};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub max_retries: u32,
    pub backoff_delay: Duration,
    pub memory_limit_mb: u64,
    pub cpu_limit_percent: u8,
    pub log_dir: PathBuf,
    pub enable_notifications: bool,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            backoff_delay: Duration::from_secs(30),
            memory_limit_mb: 500,
            cpu_limit_percent: 50,
            log_dir: PathBuf::from("logs"),
            enable_notifications: true,
        }
    }
}
