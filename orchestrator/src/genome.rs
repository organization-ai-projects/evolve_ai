use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileGene {
    pub path: String,
    pub active: bool,
    pub functions: HashMap<String, bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GenomeConfig {
    pub files: Vec<FileGene>,
}
