/// Liste simple de stopwords anglais (à enrichir)
pub fn is_stopword(word: &str) -> bool {
    matches!(
        word,
        "the" | "a" | "an" | "and" | "or" | "to" | "of" | "in" | "on" | "for" | "with"
    )
}

/// Stemming très basique : enlève 'ing', 'ed', 's' finaux
pub fn stem(word: &str) -> String {
    if word.ends_with("ing") && word.len() > 4 {
        word[..word.len() - 3].to_string()
    } else if word.ends_with("ed") && word.len() > 4 {
        word[..word.len() - 2].to_string()
    } else if word.ends_with('s') && word.len() > 3 {
        word[..word.len() - 1].to_string()
    } else {
        word.to_string()
    }
}

/// Tokenisation simple (split sur espace et ponctuation)
pub fn tokenize(text: &str) -> Vec<String> {
    text.split(|c: char| !c.is_alphanumeric())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_lowercase())
        .collect()
}

/// Calcule la fréquence de chaque mot dans un texte
pub fn word_frequencies(tokens: &[String]) -> std::collections::HashMap<String, usize> {
    let mut freq = std::collections::HashMap::new();
    for token in tokens {
        *freq.entry(token.clone()).or_insert(0) += 1;
    }
    freq
}

/// Score de similarité Jaccard entre deux ensembles de mots
pub fn jaccard_similarity(a: &[String], b: &[String]) -> f32 {
    let set_a: std::collections::HashSet<_> = a.iter().collect();
    let set_b: std::collections::HashSet<_> = b.iter().collect();
    let intersection = set_a.intersection(&set_b).count();
    let union = set_a.union(&set_b).count();
    if union == 0 {
        0.0
    } else {
        intersection as f32 / union as f32
    }
}

/// Similarité cosinus entre deux vecteurs de fréquences de mots
pub fn cosine_similarity(
    a: &std::collections::HashMap<String, usize>,
    b: &std::collections::HashMap<String, usize>,
) -> f32 {
    let mut dot = 0.0;
    let mut norm_a = 0.0;
    let mut norm_b = 0.0;
    for (k, va) in a {
        let vb = *b.get(k).unwrap_or(&0);
        dot += (*va as f32) * (vb as f32);
        norm_a += (*va as f32).powi(2);
    }
    for vb in b.values() {
        norm_b += (*vb as f32).powi(2);
    }
    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        dot / (norm_a.sqrt() * norm_b.sqrt())
    }
}

/// Génère des n-grams (n=2 ou 3) à partir d'une liste de tokens
pub fn ngrams(tokens: &[String], n: usize) -> Vec<String> {
    if tokens.len() < n {
        return Vec::new();
    }
    (0..=tokens.len() - n)
        .map(|i| tokens[i..i + n].join("_"))
        .collect()
}

/// Score qualitatif d'un message de commit (longueur, mots-clés, etc.)
pub fn score_commit_message(msg: &str) -> i32 {
    let tokens = tokenize(msg);
    let len = tokens.len() as i32;
    let important = ["fix", "feat", "add", "remove", "update", "refactor"];
    let mut score = 0;
    for word in &tokens {
        if important.iter().any(|&imp| word.contains(imp)) {
            score += 2;
        }
    }
    // Bonus pour une taille raisonnable (5 à 15 mots)
    if len >= 5 && len <= 15 {
        score += 3;
    }
    // Malus si trop court ou trop long
    if len < 3 || len > 20 {
        score -= 2;
    }
    score
}
