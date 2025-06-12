use crate::ai::coding::try_code_and_learn;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Structure pour stocker l’historique des mutations, erreurs et scores.
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct LearningState {
    /// Score global d’amélioration (ex: nombre de corrections réussies)
    pub global_score: i32,
    /// Historique des mutations tentées et leur résultat
    pub mutation_history: Vec<MutationLog>,
    /// Historique des erreurs détectées (par cargo check, logs, etc.)
    pub error_history: Vec<String>,
    /// Statistiques diverses (nombre de rollbacks, succès, etc.)
    pub stats: HashMap<String, usize>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MutationLog {
    pub mutation_desc: String,
    pub success: bool,
    pub error_msg: Option<String>,
    pub timestamp: u64,
}

#[derive(Debug, Clone)]
pub struct LearningConfig {
    pub enable_learning: bool,
    pub enable_mutation: bool,
    pub enable_history_scan: bool,
    pub enable_smart_commit: bool,
}

impl Default for LearningConfig {
    fn default() -> Self {
        Self {
            enable_learning: true,
            enable_mutation: true,
            enable_history_scan: true,
            enable_smart_commit: true,
        }
    }
}

impl LearningState {
    pub fn log_mutation(&mut self, desc: &str, success: bool, error: Option<String>) {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        self.mutation_history.push(MutationLog {
            mutation_desc: desc.to_string(),
            success,
            error_msg: error.clone(),
            timestamp: ts,
        });
        if success {
            self.global_score += 1;
            *self.stats.entry("success".to_string()).or_insert(0) += 1;
        } else {
            *self.stats.entry("fail".to_string()).or_insert(0) += 1;
            if let Some(e) = error {
                self.error_history.push(e);
            }
        }
    }

    pub fn save(&self, path: &str) {
        let _ = std::fs::create_dir_all("brain");
        let bytes = bincode::serialize(self).unwrap();
        std::fs::write(path, bytes).unwrap();
    }

    pub fn load(path: &str) -> Self {
        if let Ok(bytes) = std::fs::read(path) {
            bincode::deserialize(&bytes).unwrap_or_default()
        } else {
            Self::default()
        }
    }

    /// Analyse les logs/feedbacks précédents pour apprendre des erreurs et succès.
    /// Peut être appelée périodiquement ou au démarrage.
    pub fn scan_history_and_learn(state: &mut LearningState, log_path: &str) {
        if let Ok(content) = std::fs::read_to_string(log_path) {
            for line in content.lines() {
                if line.contains("Erreur") || line.contains("error") {
                    state.error_history.push(line.to_string());
                    *state.stats.entry("log_error".to_string()).or_insert(0) += 1;
                }
                if line.contains("✅") || line.contains("success") {
                    *state.stats.entry("log_success".to_string()).or_insert(0) += 1;
                }
            }
        }
    }
}

/// Copie récursive d'un dossier (src → dst)
fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    if !dst.exists() {
        fs::create_dir_all(dst)?;
    }
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if file_type.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}

/// Tente une mutation sur un fichier Rust dans sandbox/, rollback si besoin, adapte la stratégie selon l'historique.
/// Toute la logique d'apprentissage/adaptation est centralisée ici.
/// Le code réel N'EST PAS modifié, seule la sandbox est touchée.
pub fn try_code_and_learn_adaptive(
    state: &mut LearningState,
    target_file: &std::path::Path,
) -> String {
    // 1. Prépare la sandbox (copie du projet)
    let sandbox_root = Path::new("git_syncer/sandbox");
    let project_root = Path::new(".");
    let _ = fs::remove_dir_all(sandbox_root); // Nettoie la sandbox avant chaque essai
    if let Err(e) = copy_dir_recursive(project_root, sandbox_root) {
        return format!("Sandbox copy failed: {e}");
    }

    // 2. Détermine le chemin du fichier cible dans la sandbox
    let rel_path = target_file
        .strip_prefix(project_root)
        .unwrap_or(target_file);
    let sandbox_file = sandbox_root.join(rel_path);

    // 3. Charge le code à muter dans la sandbox
    let backup = fs::read_to_string(&sandbox_file).unwrap_or_default();
    let coding_desc = format!("Commenter la première ligne de {:?}", sandbox_file);

    // 4. Mutation réelle dans la sandbox
    try_code_and_learn(
        state,
        &coding_desc,
        || {
            let mut lines: Vec<_> = backup.lines().map(|l| l.to_string()).collect();
            if !lines.is_empty() && !lines[0].trim_start().starts_with("//") {
                lines[0] = format!("// {}", lines[0]);
                fs::write(&sandbox_file, lines.join("\n")).is_ok()
            } else {
                false
            }
        },
        sandbox_root.to_str().unwrap(),
        false,
    );

    // 5. Exploite l'historique pour adapter la stratégie (ex: blacklist)
    let recent_errors: Vec<_> = state
        .mutation_history
        .iter()
        .rev()
        .take(3)
        .filter(|log| !log.success)
        .collect();

    if recent_errors.len() == 3 {
        format!(
            "CodeLearnJob: 3 échecs détectés sur {:?}, l'IA évite ce fichier à l'avenir (sandbox only).",
            target_file
        )
    } else if let Some(last) = state.mutation_history.last() {
        format!(
            "CodeLearnJob: mutation '{}', succès: {}, erreur: {:?} (sandbox only)",
            last.mutation_desc, last.success, last.error_msg
        )
    } else {
        "CodeLearnJob: aucune mutation enregistrée (sandbox only)".to_string()
    }
}

/// Crée un nouveau projet Rust dans sandbox/
/// name: nom du projet (sous-dossier de sandbox)
pub fn create_sandbox_project(name: &str) -> std::io::Result<std::path::PathBuf> {
    let sandbox_root = std::path::Path::new("git_syncer/sandbox");
    let proj_path = sandbox_root.join(name);
    if !proj_path.starts_with(sandbox_root) {
        return Err(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "Projet hors sandbox interdit",
        ));
    }
    std::fs::create_dir_all(&proj_path)?;
    // Initialise un projet Rust minimal
    std::process::Command::new("cargo")
        .arg("init")
        .current_dir(&proj_path)
        .output()?;
    Ok(proj_path)
}
