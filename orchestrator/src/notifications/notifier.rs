use crate::agent_listing::AgentInfo;

enum NotificationType {
    AgentKilled { reason: String },
    AgentCrashed { error: String },
    ResourceLimit { resource: String, value: u64 },
}

// Fonctions publiques exposées
pub fn notify_killed(agent: &AgentInfo, reason: String) {
    notify(agent, NotificationType::AgentKilled { reason });
}

pub fn notify_crashed(agent: &AgentInfo, error: String) {
    notify(agent, NotificationType::AgentCrashed { error });
}

pub fn notify_resource_limit(agent: &AgentInfo, resource: String, value: u64) {
    notify(agent, NotificationType::ResourceLimit { resource, value });
}

pub fn notify_disabled(agent: &AgentInfo, reason: String) {
    notify(agent, NotificationType::AgentKilled { reason });
}

pub fn notify_event(agent: &AgentInfo, event: &str) {
    println!("🔔 Event: {} pour agent {}", event, agent.name);
}

// Fonction interne
fn notify(agent: &AgentInfo, notification: NotificationType) {
    // TODO: Implémenter les différents canaux (Discord, mail...)
    match notification {
        NotificationType::AgentKilled { reason } => {
            println!("🚨 Agent {} killed: {}", agent.name, reason)
        }
        NotificationType::AgentCrashed { error } => {
            println!("💥 Agent {} crashed: {}", agent.name, error)
        }
        NotificationType::ResourceLimit { resource, value } => {
            println!("⚠️ Agent {} exceeded {}: {}", agent.name, resource, value)
        }
    }
}
