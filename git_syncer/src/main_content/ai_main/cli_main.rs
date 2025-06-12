use super::handle_ai_mode;
use crate::all_access::channel;
use crate::cli::Args;

/// Handler CLI IA : route vers handle_ai_mode sans interaction humaine directe.
pub fn handle_ai_cli(args: &Args) -> bool {
    handle_ai_mode(args)
}

/// Pour envoyer un message à une autre entité (ex: à l'humain)
pub fn ia_send_message_to_human(msg: &str) {
    channel::send_message("ia", "human", msg);
}

/// Pour lire les messages envoyés par l'humain à l'IA
pub fn ia_read_messages_from_human() -> Vec<String> {
    channel::read_messages("human", "ia")
}
