use rand::Rng;

pub fn mutate_character(code: &str) -> String {
    let mut rng = rand::thread_rng();
    let mut result = String::from(code);

    // Type de mutation: 0=modification, 1=suppression, 2=mutation byte
    match rng.gen_range(0..3) {
        0 => {
            // Mutation par modification
            if let Some(position) = (0..code.len()).choose(&mut rng) {
                let new_char = match rng.gen_range(0..3) {
                    0 => ('a'..='z').choose(&mut rng).unwrap(),
                    1 => ('0'..='9').choose(&mut rng).unwrap(),
                    _ => ['(', ')', '{', '}', '[', ']', '.', ';']
                        .choose(&mut rng)
                        .unwrap(),
                };

                result.replace_range(position..position + 1, &new_char.to_string());
            }
        }
        1 => {
            // Mutation par suppression
            if let Some(position) = (0..code.len()).choose(&mut rng) {
                result.remove(position);
            }
        }
        2 => {
            // Mutation au niveau byte
            if let Some(position) = (0..code.len()).choose(&mut rng) {
                let mut bytes = result.into_bytes();
                if !bytes.is_empty() {
                    // Flip un bit aléatoire dans le byte
                    let bit = rng.gen_range(0..8);
                    bytes[position] ^= 1 << bit;
                    // Convertir en gardant les caractères valides UTF-8
                    result = String::from_utf8_lossy(&bytes).into_owned();
                }
            }
        }
        _ => {}
    }
    result
}
