use crate::agent_structural_code::AgentStructuralCode;
use serde::Serialize; // Import du trait Serialize
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize)] // Ajout du trait Serialize
pub struct AgentScanResult {
    pub code_hash: String,
    pub file_hashes: HashMap<String, String>,
    pub files: HashMap<String, String>,
    pub file_metrics: HashMap<String, AgentStructuralCode>,
}

pub struct RustScanner {
    base_path: PathBuf,
}

impl RustScanner {
    pub fn new(base_path: PathBuf) -> Self {
        Self { base_path }
    }

    /// Scan le template et retourne le résultat
    /// C'est une fonction utilitaire pour éviter de dupliquer la logique
    /// dans d'autres parties du code
    pub fn scan_template(&self, template_dir: &Path) -> std::io::Result<AgentScanResult> {
        self.scan_agent(template_dir, false)
    }

    /// Scan complet d'un agent
    /// - Scanne tous les fichiers .rs
    /// - Calcule les hashes (global et par fichier)
    /// - Analyse les métriques de code
    /// - Persiste optionnellement le résultat
    pub fn scan_agent(&self, agent_path: &Path, persist: bool) -> std::io::Result<AgentScanResult> {
        let mut files = HashMap::new();
        let mut file_hashes = HashMap::new();
        let mut file_metrics = HashMap::new();

        for file_path in self.scan_rust_files(agent_path)? {
            let content = fs::read_to_string(&file_path)?;
            let rel_path = file_path
                .strip_prefix(agent_path)
                .unwrap_or(&file_path)
                .to_string_lossy()
                .to_string();

            files.insert(rel_path.clone(), content.clone());
            let hash = format!("{:x}", Sha256::digest(content.as_bytes()));
            file_hashes.insert(rel_path.clone(), hash);
            let metrics = crate::agent_code_analysis::analyze_structural_code(&file_path);
            file_metrics.insert(rel_path, metrics);
        }

        let code_hash = {
            let mut hasher = Sha256::new();
            for content in files.values() {
                hasher.update(content.as_bytes());
            }
            format!("{:x}", hasher.finalize())
        };

        let result = AgentScanResult {
            code_hash,
            file_hashes,
            files,
            file_metrics,
        };

        if persist {
            let persist_path = agent_path.join("scan_result.bin");
            let bytes = bincode::serialize(&result).unwrap();
            fs::write(persist_path, bytes)?;
        }

        Ok(result)
    }

    /// Scan une liste de fichiers .rs
    fn scan_rust_files(&self, path: &Path) -> std::io::Result<Vec<PathBuf>> {
        let mut rust_files = Vec::new();
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                rust_files.extend(self.scan_rust_files(&path)?);
            } else if path.extension().map_or(false, |ext| ext == "rs") {
                rust_files.push(path);
            }
        }
        Ok(rust_files)
    }
}
