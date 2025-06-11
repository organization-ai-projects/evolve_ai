const DANGEROUS_PATTERNS: &[&str] = &[
    "std::process::Command",
    "std::env::var",
    "unsafe",
    "asm!",
    "std::ptr",
    // Ajouter d'autres patterns dangereux
];

/// Vérifie si le contenu d'un fichier est sûr
/// Autorise les actions sur le propre dossier de l'IA
fn is_content_safe(content: &str, agent_path: &str) -> bool {
    find_dangerous_pattern(content).is_none() && validate_code_structure(content, agent_path)
}

/// Nettoie le contenu d'un fichier (supprime les lignes dangereuses et écrit sur disque)
/// Retourne `true` si le nettoyage et l'écriture ont réussi, `false` sinon.
pub fn sanitize_code(path: &str, content: &str, agent_path: &str) -> bool {
    let cleaned_content = content
        .lines()
        .filter(|line| find_dangerous_pattern(line).is_none() || line.contains(agent_path))
        .collect::<Vec<_>>()
        .join("\n");

    match std::fs::write(path, cleaned_content) {
        Ok(_) => true,
        Err(e) => {
            eprintln!(
                "Erreur lors de l'écriture du fichier nettoyé {}: {}",
                path, e
            );
            false
        }
    }
}

/// Vérifie si tous les fichiers donnés sont sûrs (scan, nettoyage si besoin, re-scan)
/// files: Vec<(chemin, contenu)>
pub fn is_code_safe(files: &[(String, String)], agent_path: &str) -> bool {
    for (path, content) in files {
        if !is_content_safe(content, agent_path) {
            // Nettoie le contenu et écrit sur disque
            if !sanitize_code(path, content, agent_path) {
                return false; // Échec du nettoyage
            }
            // Re-scan après nettoyage
            if !is_content_safe(content, agent_path) {
                return false; // Toujours dangereux après nettoyage
            }
        }
    }
    true // Tous les fichiers sont sûrs
}

fn find_dangerous_pattern(content: &str) -> Option<&'static str> {
    DANGEROUS_PATTERNS
        .iter()
        .find(|&&pattern| content.contains(pattern))
        .copied()
}

fn validate_code_structure(content: &str, agent_path: &str) -> bool {
    !content
        .lines()
        .filter(|line| line.trim().starts_with("use "))
        .any(|line| {
            DANGEROUS_PATTERNS
                .iter()
                .any(|pattern| line.contains(pattern) && !line.contains(agent_path))
        })
}
