use crate::ai::nlp::{
    cosine_similarity, jaccard_similarity, score_commit_message, word_frequencies,
};
use rand::prelude::SliceRandom;
use rand::Rng;
use ron::de::from_str;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct CommitPattern {
    pub file_keywords: Vec<String>,
    pub op_types: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CommitNeuron {
    pub pattern: CommitPattern,
    pub message: String,
    pub weight: f32,
    pub score: i32,
    pub seen: u32,
    pub fails: u32,
    pub last_success: Option<u64>, // timestamp unix
                                   // pub context: Option<String>, // à activer si tu veux stocker plus d'infos
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct CommitBrain {
    pub neurons: Vec<CommitNeuron>,
    pub action_stats: HashMap<String, u32>,
    pub history: Vec<String>, // Historique d’actions/messages
}

impl CommitBrain {
    pub fn predict_msg(&self, pattern: &CommitPattern) -> Option<&CommitNeuron> {
        let freq = word_frequencies(&pattern.file_keywords);
        self.neurons.iter().max_by(|a, b| {
            // Similarité cosinus sur les fréquences de mots
            let freq_a = word_frequencies(&a.pattern.file_keywords);
            let freq_b = word_frequencies(&b.pattern.file_keywords);
            let cos_a = cosine_similarity(&freq_a, &freq);
            let cos_b = cosine_similarity(&freq_b, &freq);

            // Score combiné : poids + cosinus + jaccard
            let sim_a = jaccard_similarity(&a.pattern.file_keywords, &pattern.file_keywords);
            let sim_b = jaccard_similarity(&b.pattern.file_keywords, &pattern.file_keywords);

            (a.weight + cos_a + sim_a)
                .partial_cmp(&(b.weight + cos_b + sim_b))
                .unwrap()
                .then_with(|| a.last_success.cmp(&b.last_success))
        })
    }

    pub fn learn_msg(&mut self, pattern: CommitPattern, msg: String, success: bool) {
        use std::time::{SystemTime, UNIX_EPOCH};
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        if let Some(neuron) = self
            .neurons
            .iter_mut()
            .find(|n| n.pattern == pattern && n.message == msg)
        {
            neuron.seen += 1;
            if success {
                neuron.weight += 1.0;
                neuron.score += 1;
                neuron.last_success = Some(now);
            } else {
                neuron.weight -= 1.0;
                neuron.fails += 1;
                neuron.score -= 2;
            }
        } else {
            self.neurons.push(CommitNeuron {
                pattern,
                message: msg,
                weight: if success { 1.0 } else { -1.0 },
                score: if success { 1 } else { -2 },
                seen: 1,
                fails: if success { 0 } else { 1 },
                last_success: if success { Some(now) } else { None },
                // context: None,
            });
        }
    }

    /// MUTATION: supprime les neurons les moins efficaces
    pub fn natural_selection(&mut self) {
        // On vire ceux qui ont trop d’échecs ou un score très bas
        let before = self.neurons.len();
        self.neurons.retain(|n| n.score > -5 && n.fails < 10);
        let after = self.neurons.len();
        if before != after {
            println!("🦠 Natural selection: {before} -> {after} neurons");
        }
    }

    /// MUTATION: crée de nouveaux neurones random
    pub fn mutate(&mut self, vocabulary: &[String]) {
        let mut rng = rand::thread_rng();
        if rng.gen_bool(0.3) && !vocabulary.is_empty() {
            let w1 = vocabulary.choose(&mut rng).unwrap().clone();
            let op = if rng.gen_bool(0.5) { "add" } else { "mod" }.to_string();
            let msg = format!("{} {}", op, w1);
            let score = score_commit_message(&msg);
            self.neurons.push(CommitNeuron {
                pattern: CommitPattern {
                    file_keywords: vec![w1],
                    op_types: vec![op],
                },
                message: msg,
                weight: 0.5 + score as f32 * 0.1,
                score,
                seen: 0,
                fails: 0,
                last_success: None,
                // context: None,
            });
        }
    }

    pub fn action_success(&mut self, action: &str) {
        *self.action_stats.entry(action.to_string()).or_insert(0) += 1;
    }

    pub fn save(&self, path: &str) {
        let dir = std::path::Path::new(path).parent().unwrap();
        let _ = std::fs::create_dir_all(dir);
        let bytes = bincode::serialize(self).unwrap();
        std::fs::write(path, bytes).unwrap();
    }

    pub fn load(path: &str) -> Self {
        if let Ok(bytes) = std::fs::read(path) {
            bincode::deserialize(&bytes).unwrap_or_default()
        } else {
            Self::default()
        }
    }

    /// Injecte des exemples de commits humains (un par ligne, format: "keywords|op_types|message")
    pub fn inject_examples(&mut self, path: &str) {
        if let Ok(content) = std::fs::read_to_string(path) {
            for line in content.lines() {
                let parts: Vec<_> = line.split('|').collect();
                if parts.len() == 3 {
                    let file_keywords: Vec<String> =
                        parts[0].split(',').map(|s| s.trim().to_string()).collect();
                    let op_types: Vec<String> =
                        parts[1].split(',').map(|s| s.trim().to_string()).collect();
                    let message = parts[2].trim().to_string();
                    self.neurons.push(CommitNeuron {
                        pattern: CommitPattern {
                            file_keywords,
                            op_types,
                        },
                        message,
                        weight: 1.0,
                        score: score_commit_message(parts[2].trim()),
                        seen: 1,
                        fails: 0,
                        last_success: None,
                    });
                }
            }
        }
    }

    /// Injecte des exemples de commits depuis un fichier RON
    pub fn inject_examples_ron(&mut self, path: &str) {
        if let Ok(content) = fs::read_to_string(path) {
            if let Ok(examples) = from_str::<Vec<CommitNeuron>>(&content) {
                for neuron in examples {
                    self.neurons.push(neuron);
                }
            }
        }
    }
}
