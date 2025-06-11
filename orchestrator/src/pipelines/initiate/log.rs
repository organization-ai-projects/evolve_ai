use std::fs;
use std::path::Path;

pub fn write_initialization_log<P: AsRef<Path>>(
    log_path: P,
    log_lines: &[String],
) -> std::io::Result<()> {
    fs::write(log_path, log_lines.join("\n"))
}
