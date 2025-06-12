use serde::Serialize;
use std::fs::OpenOptions;
use std::io::Write;

pub fn ia_log(msg: &str, log_dir: &str) {
    let date = chrono::Local::now().format("%Y-%m-%d").to_string();
    let log_path = format!("{}/ia_{}.log", log_dir, date);
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .unwrap();
    let ts = chrono::Local::now().to_rfc3339();
    writeln!(file, "[{ts}] {msg}").ok();
}

#[derive(Serialize)]
pub struct IaJobLog<'a> {
    pub timestamp: String,
    pub job: &'a str,
    pub success: bool,
    pub message: &'a str,
    pub duration_ms: u128,
}

pub fn ia_log_json(job: &str, msg: &str, success: bool, log_dir: &str) {
    let date = chrono::Local::now().format("%Y-%m-%d").to_string();
    let log_path = format!("{}/ia_{}.jsonl", log_dir, date);
    let log = IaJobLog {
        timestamp: chrono::Local::now().to_rfc3339(),
        job,
        success,
        message: msg,
        duration_ms: 0,
    };
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .unwrap();
    let _ = writeln!(file, "{}", serde_json::to_string(&log).unwrap());
}
