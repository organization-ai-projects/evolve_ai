use quote::ToTokens;
use rand::Rng;
use syn::{parse_file, File, Item};

pub fn mutate_ast(code: &str) -> Result<String, syn::Error> {
    let mut rng = rand::thread_rng();

    match rng.gen_range(0..3) {
        0 => {
            // Mutation AST classique
            let mut ast = parse_file(code)?;
            if let Some(idx) = (0..ast.items.len()).choose(&mut rng) {
                match &ast.items[idx] {
                    Item::Fn(func) => match rng.gen_range(0..3) {
                        0 => ast.items.remove(idx),
                        1 => ast.items.push(ast.items[idx].clone()),
                        _ => ast.items.swap(idx, rng.gen_range(0..ast.items.len())),
                    },
                    _ => {}
                }
            }
            Ok(ast.into_token_stream().to_string())
        }
        1 => {
            // Mutation de structure
            let mut ast = parse_file(code)?;
            match rng.gen_range(0..3) {
                0 => Ok("// Code temporairement désactivé\n".to_string()),
                1 => Ok(code.replace("pub ", "")),
                _ => Ok(format!("#[derive(Debug)]\n{}", code)),
            }
        }
        2 => {
            // Mutation byte
            let mut bytes = code.as_bytes().to_vec();
            if !bytes.is_empty() {
                match rng.gen_range(0..3) {
                    0 => bytes[0] ^= 1 << rng.gen_range(0..8),
                    1 => bytes.push(b';'),
                    _ => bytes.extend_from_slice(b"/* mutation */"),
                }
                Ok(String::from_utf8_lossy(&bytes).into_owned())
            } else {
                Ok(code.to_string())
            }
        }
    }
}
