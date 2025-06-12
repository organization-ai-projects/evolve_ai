use super::ia_bridge::generate_code_ia;

/// Direction de la demande (d’où vers où)
pub enum BridgeDirection {
    HumanToIa,
    IaToHuman,
    // d’autres si besoin
}

/// Route une demande de génération de code selon le contexte
pub fn route_generate_code(prompt: &str, direction: BridgeDirection) -> String {
    match direction {
        BridgeDirection::HumanToIa => {
            // Ici tu logs, traces, ajoutes sécurité, etc.
            generate_code_ia(prompt)
        }
        BridgeDirection::IaToHuman => {
            // Ici, tu pourrais brancher un bridge humain si tu veux une “réponse” humaine
            // (rare, mais utile pour logs, validation, ou apprentissage interactif)
            unimplemented!()
        }
    }
}
