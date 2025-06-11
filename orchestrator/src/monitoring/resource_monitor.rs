use std::process::Child;
use sysinfo::{ProcessExt, System, SystemExt};

/// Surveillance pure des ressources système, sans dépendance à la config
/// Retourne true si le process dépasse une des limites données
pub fn check_resource_usage(process: &Child, memory_limit_mb: u64, cpu_limit_percent: u8) -> bool {
    let sys = System::new_all();
    let pid = sysinfo::Pid::from(process.id() as usize);

    if let Some(sys_process) = sys.process(pid) {
        let memory_mb = sys_process.memory() / 1024 / 1024;
        let cpu_percent = sys_process.cpu_usage() as u8;
        memory_mb > memory_limit_mb || cpu_percent > cpu_limit_percent
    } else {
        false
    }
}
