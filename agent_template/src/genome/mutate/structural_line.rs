use rand::Rng;

pub fn mutate_line(code: &str) -> String {
    let mut lines: Vec<&str> = code.lines().collect();
    let mut rng = rand::thread_rng();

    // Stratégie principale de mutation
    match rng.gen_range(0..3) {
        0 => {
            // Mutation structurelle de ligne
            match rng.gen_range(0..3) {
                0 => {
                    // Suppression d'une ligne
                    if !lines.is_empty() {
                        lines.remove(rng.gen_range(0..lines.len()));
                    }
                }
                1 => {
                    // Duplication d'une ligne
                    if !lines.is_empty() {
                        let idx = rng.gen_range(0..lines.len());
                        let line = lines[idx];
                        lines.insert(idx, line);
                    }
                }
                _ => {
                    // Échange de deux lignes
                    if lines.len() >= 2 {
                        let idx1 = rng.gen_range(0..lines.len());
                        let idx2 = rng.gen_range(0..lines.len());
                        lines.swap(idx1, idx2);
                    }
                }
            }
        }
        1 => {
            // Mutation de contenu de ligne
            if !lines.is_empty() {
                let idx = rng.gen_range(0..lines.len());
                lines[idx] = match rng.gen_range(0..3) {
                    0 => "    ",                        // Indentation
                    1 => "// TODO: Code à implémenter", // Commentaire
                    _ => "",                            // Ligne vide
                };
            }
        }
        2 => {
            // Mutation byte sur ligne
            if !lines.is_empty() {
                let idx = rng.gen_range(0..lines.len());
                let mut bytes = lines[idx].as_bytes().to_vec();
                if !bytes.is_empty() {
                    match rng.gen_range(0..3) {
                        0 => {
                            let bit = rng.gen_range(0..8);
                            bytes[0] ^= 1 << bit;
                        }
                        1 => {
                            let pos = rng.gen_range(0..bytes.len());
                            bytes[pos] = rng.gen::<u8>();
                        }
                        _ => bytes.push(rng.gen::<u8>()),
                    }
                    if let Ok(new_line) = String::from_utf8(bytes) {
                        lines[idx] = Box::leak(new_line.into_boxed_str());
                    }
                }
            }
        }
    }

    lines.join("\n")
}
