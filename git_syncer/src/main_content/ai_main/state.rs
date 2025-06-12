use chrono::Utc;
use std::fs::{self};
use std::io::Write;

pub struct IaStateManager {
    pub state_path: &'static str,
    pub lock_path: &'static str,
    pub log_dir: &'static str,
    pub undo_log: &'static str,
    pub cycles: usize,
    pub success: usize,
    pub fail: usize,
    pub start_time: chrono::DateTime<Utc>,
}

impl IaStateManager {
    pub fn new() -> Self {
        Self {
            state_path: "git_syncer/.ia_state",
            lock_path: "git_syncer/.ia_state.lock",
            log_dir: "git_syncer/ia_logs",
            undo_log: "git_syncer/ia_logs/undo.log",
            cycles: 0,
            success: 0,
            fail: 0,
            start_time: Utc::now(),
        }
    }
    pub fn log_dir(&self) -> &str {
        self.log_dir
    }
    pub fn try_lock(&self) -> bool {
        if fs::metadata(self.lock_path).is_ok() {
            false
        } else {
            fs::write(self.lock_path, "locked").is_ok()
        }
    }
    pub fn init_running(&self) {
        fs::write(
            self.state_path,
            "running\ncycles:0\nlast:\nsuccess:0\nfail:0\nheartbeat:\n",
        )
        .ok();
    }
    pub fn handle_cycle(&mut self, results: &[Result<String, String>]) {
        let mut success = 0;
        let mut fail = 0;
        for res in results {
            if res.is_ok() {
                success += 1;
            } else {
                fail += 1;
            }
        }
        self.cycles += 1;
        self.success += success;
        self.fail += fail;
        let last_cycle = Utc::now().to_rfc3339();
        let uptime = (Utc::now() - self.start_time).num_seconds();
        let state = format!(
            "running\ncycles:{}\nlast:{}\nsuccess:{}\nfail:{}\nuptime:{}\nheartbeat:{}",
            self.cycles,
            last_cycle,
            self.success,
            self.fail,
            uptime,
            Utc::now().to_rfc3339()
        );
        fs::write(self.state_path, state).ok();
    }
    pub fn finalize(&self) {
        let last_cycle = Utc::now().to_rfc3339();
        let uptime = (Utc::now() - self.start_time).num_seconds();
        fs::write(
            self.state_path,
            format!(
                "stopped\ncycles:{}\nlast:{}\nsuccess:{}\nfail:{}\nuptime:{}\nheartbeat:{}",
                self.cycles,
                last_cycle,
                self.success,
                self.fail,
                uptime,
                Utc::now().to_rfc3339()
            ),
        )
        .ok();
        let _ = fs::remove_file(self.lock_path);
    }
    pub fn stop(&self) {
        fs::write(self.state_path, "stopped").ok();
        let _ = fs::remove_file(self.lock_path);
        println!("🛑 Demande d'arrêt de l'IA (flag .ia_state mis à jour)");
    }
    pub fn print_status(&self) {
        let status = if fs::read_to_string(self.state_path)
            .unwrap_or_default()
            .contains("running")
        {
            "🟢 IA running"
        } else {
            "🔴 IA stopped"
        };
        println!("{status}");
        if let Ok(state) = fs::read_to_string(self.state_path) {
            println!("Dernier état IA : {state}");
        }
        if let Ok(state) = fs::read_to_string(self.state_path) {
            for line in state.lines() {
                if line.starts_with("cycles:") || line.starts_with("last:") {
                    println!("{line}");
                }
            }
        }
    }
    pub fn self_update(&self) {
        println!("🔄 Mise à jour du projet (git pull + rebuild)...");
        let pull_ok = std::process::Command::new("git")
            .args(["pull"])
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
        let build_status = std::process::Command::new("cargo").args(["build"]).status();
        match build_status {
            Ok(s) if s.success() && pull_ok => println!("✅ Orchestrateur à jour !"),
            _ => {
                eprintln!("❌ Build échoué, orchestrateur non rebuild !");
                super::log::ia_log("❌ Build échoué lors du self-update", self.log_dir);
            }
        }
    }
    pub fn undo_last_action(&self) -> bool {
        if let Ok(log) = fs::read_to_string(self.undo_log) {
            if let Some(last) = log.lines().last() {
                if let Some(commit_hash) = last.split('|').nth(1) {
                    println!("⏪ Undo IA : git reset --hard {commit_hash}");
                    let _ = std::process::Command::new("git")
                        .args(["reset", "--hard", commit_hash])
                        .status();
                    return true;
                }
            }
        }
        false
    }
    /// Log le hash du commit courant dans le undo log (pour undo IA)
    pub fn log_undo_commit(&self) {
        if let Ok(output) = std::process::Command::new("git")
            .args(["rev-parse", "HEAD"])
            .output()
        {
            let hash = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let undo_entry = format!("{}|{}", chrono::Utc::now().to_rfc3339(), hash);
            if let Ok(mut file) = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(self.undo_log)
            {
                let _ = writeln!(file, "{undo_entry}");
            }
        }
    }
}
