use super::ast::{extract_ast_features, extract_syn_features};
use super::nlp::{is_stopword, ngrams, stem, tokenize};
use super::semantics::map_rust_type;
use regex::Regex;

pub fn extract_features(diff: &str) -> (Vec<String>, Vec<String>) {
    let mut keywords = Vec::new();
    let mut ops = Vec::new();

    let re_add = Regex::new(r"^\+\s*(\w+)").unwrap();
    let re_rm = Regex::new(r"^-\s*(\w+)").unwrap();
    let re_file = Regex::new(r"^diff --git a/(\S+) b/(\S+)").unwrap();
    let re_fn = Regex::new(r"^\+\s*fn\s+(\w+)").unwrap();
    let re_struct = Regex::new(r"^\+\s*struct\s+(\w+)").unwrap();
    let re_enum = Regex::new(r"^\+\s*enum\s+(\w+)").unwrap();

    for line in diff.lines() {
        // Fichiers impactés
        if let Some(cap) = re_file.captures(line) {
            let fname = cap[1].split('/').last().unwrap().to_lowercase();
            for token in tokenize(&fname) {
                let token = stem(&token);
                if !is_stopword(&token) && !keywords.contains(&token) {
                    keywords.push(token);
                }
            }
        }
        // Noms de fonctions ajoutées
        if let Some(cap) = re_fn.captures(line) {
            let fn_name = cap[1].to_lowercase();
            if !keywords.contains(&fn_name) {
                keywords.push(fn_name);
            }
        }
        // Types manipulés (struct/enum)
        if let Some(cap) = re_struct.captures(line) {
            let struct_name = cap[1].to_lowercase();
            if !keywords.contains(&struct_name) {
                keywords.push(struct_name);
            }
        }
        if let Some(cap) = re_enum.captures(line) {
            let enum_name = cap[1].to_lowercase();
            if !keywords.contains(&enum_name) {
                keywords.push(enum_name);
            }
        }
        // Patterns de diff
        if let Some(cap) = re_add.captures(line) {
            let tokens = tokenize(&cap[1].to_lowercase());
            for token in &tokens {
                let token = stem(token);
                if !is_stopword(&token) && !keywords.contains(&token) {
                    keywords.push(token);
                }
            }
            // Ajoute les n-grams (2 et 3)
            for n in 2..=3 {
                for ng in ngrams(&tokens, n) {
                    if !keywords.contains(&ng) {
                        keywords.push(ng);
                    }
                }
            }
            if !ops.contains(&"add".to_string()) {
                ops.push("add".to_string());
            }
        }
        if let Some(cap) = re_rm.captures(line) {
            let tokens = tokenize(&cap[1].to_lowercase());
            for token in &tokens {
                let token = stem(token);
                if !is_stopword(&token) && !keywords.contains(&token) {
                    keywords.push(token);
                }
            }
            for n in 2..=3 {
                for ng in ngrams(&tokens, n) {
                    if !keywords.contains(&ng) {
                        keywords.push(ng);
                    }
                }
            }
            if !ops.contains(&"remove".to_string()) {
                ops.push("remove".to_string());
            }
        }
    }
    // Ajoute les features AST extraites du diff complet (regex)
    for ast_feat in extract_ast_features(diff) {
        if !keywords.contains(&ast_feat) {
            keywords.push(ast_feat);
        }
    }
    // Ajoute les features AST extraites par syn (plus précises)
    for syn_feat in extract_syn_features(diff) {
        // Si la feature commence par "fn:arg:" ou "fn:return:", applique le mapping sémantique
        if let Some(rest) = syn_feat.strip_prefix("fn:arg:") {
            let mapped = map_rust_type(rest);
            if !keywords.contains(&mapped) {
                keywords.push(mapped);
            }
        } else if let Some(rest) = syn_feat.strip_prefix("fn:return:") {
            let mapped = map_rust_type(rest);
            if !keywords.contains(&mapped) {
                keywords.push(mapped);
            }
        } else if !keywords.contains(&syn_feat) {
            keywords.push(syn_feat);
        }
    }

    if ops.is_empty() {
        ops.push("mod".to_string());
    }
    (keywords, ops)
}
