use std::fs;
use std::path::Path;

pub fn update_cargo_toml(agent_dir: &Path, agent_id: &str) -> std::io::Result<()> {
    let cargo_path = agent_dir.join("Cargo.toml");
    let mut cargo_content = fs::read_to_string(&cargo_path)?;
    cargo_content = cargo_content.replace("{agent_name}", agent_id);
    fs::write(&cargo_path, cargo_content)?;
    Ok(())
}
