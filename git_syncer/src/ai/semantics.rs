use std::collections::HashMap;
use std::sync::OnceLock;

static TYPE_MAPPING: OnceLock<HashMap<String, String>> = OnceLock::new();

fn load_type_mapping() -> HashMap<String, String> {
    let path = "git_syncer/brain/code/rust/type_mapping.ron";
    if let Ok(content) = std::fs::read_to_string(path) {
        if let Ok(map) = ron::from_str::<HashMap<String, String>>(&content) {
            return map;
        }
    }
    HashMap::new()
}

/// Mapping simplifié des types Rust vers des concepts sémantiques
pub fn map_rust_type(ty: &str) -> String {
    let mapping = TYPE_MAPPING.get_or_init(load_type_mapping);
    let ty = ty.trim();
    for (k, v) in mapping {
        if k == "<generic>" && ty.contains('<') && ty.contains('>') {
            return v.clone();
        }
        if ty.starts_with(k) {
            return v.clone();
        }
    }
    ty.to_string()
}
