use crate::agent_structural_code::AgentStructuralCode;
use std::path::Path;

pub fn analyze_structural_code(file_path: &Path) -> AgentStructuralCode {
    let content = std::fs::read_to_string(file_path).unwrap_or_default();

    AgentStructuralCode {
        unwrap_count: content.matches(".unwrap(").count() as u32,
    }
}
