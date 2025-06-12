/// Reçoit une demande de génération de code et appelle l’IA.
/// Ne sait rien du CLI ni de l’humain.
pub fn generate_code_ia(prompt: &str) -> String {
    crate::ai::coding::generate_code_from_prompt(prompt)
}

// Ajoute ici d’autres APIs IA au besoin
// pub fn refactor_code_ia(...)
// pub fn repair_code_ia(...)
