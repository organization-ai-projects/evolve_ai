use crate::ai::semantics::map_rust_type;
use regex::Regex;
use syn::visit::Visit;

/// Extrait des features AST avancées et typées depuis un diff ou un fichier source Rust
pub fn extract_ast_features(code: &str) -> Vec<String> {
    let mut features = Vec::new();

    let re_fn = Regex::new(r"\b(pub\s+)?fn\s+(\w+)\s*\(([^)]*)\)\s*(->\s*[\w:<>,&\s]+)?").unwrap();
    let re_struct = Regex::new(r"\b(pub\s+)?struct\s+(\w+)(<[^>]+>)?").unwrap();
    let re_struct_field = Regex::new(r"^\s*(pub\s+)?(\w+):\s*([^,}]+)").unwrap();
    let re_enum = Regex::new(r"\b(pub\s+)?enum\s+(\w+)(<[^>]+>)?").unwrap();
    let re_enum_variant = Regex::new(r"^\s*(\w+)\s*(\(|\{|,|$)").unwrap();
    let re_trait = Regex::new(r"\b(pub\s+)?trait\s+(\w+)(<[^>]+>)?").unwrap();
    let re_impl = Regex::new(r"\bimpl(\s*<[^>]+>)?\s+([\w:<>]+)").unwrap();
    let re_macro = Regex::new(r"(\w+)!").unwrap();
    let re_attr = Regex::new(r"^#\[(\w+)").unwrap();
    let re_mod = Regex::new(r"\bmod\s+(\w+)").unwrap();
    let re_use = Regex::new(r"\buse\s+([\w:_:]+)").unwrap();
    let re_test = Regex::new(r"#\s*\[\s*test\s*]").unwrap();

    // Fonctions
    for cap in re_fn.captures_iter(code) {
        let name = &cap[2];
        let args = cap.get(3).map(|m| m.as_str()).unwrap_or("");
        let ret = cap.get(4).map(|m| m.as_str()).unwrap_or("");
        features.push(format!("fn:{}", name));
        if cap.get(1).is_some() {
            features.push("fn:pub".to_string());
        }
        if !args.is_empty() {
            for arg in args.split(',') {
                let arg = arg.trim();
                if arg.starts_with("mut ") {
                    features.push("fn:arg_mut".to_string());
                }
                if let Some((name, ty)) = arg.split_once(':') {
                    let mapped = map_rust_type(ty.trim());
                    features.push(format!("fn:arg_type:{}", mapped));
                    features.push(format!("fn:arg_name:{}", name.trim()));
                }
            }
        }
        if !ret.is_empty() {
            let ret_clean = ret.replace("->", "");
            let ret_ty = ret_clean.trim();
            let mapped = map_rust_type(ret_ty);
            features.push(format!("fn:return_type:{}", mapped));
        }
    }

    // Structs et champs
    for cap in re_struct.captures_iter(code) {
        let name = &cap[2];
        features.push(format!("struct:{}", name));
        if cap.get(1).is_some() {
            features.push("struct:pub".to_string());
        }
        if let Some(generics) = cap.get(3) {
            features.push(format!("struct:generic:{}", generics.as_str()));
        }
    }
    for cap in re_struct_field.captures_iter(code) {
        let field = &cap[2];
        let ty = &cap[3];
        let mapped = map_rust_type(ty);
        features.push(format!("struct:field:{}", field));
        features.push(format!("struct:field_type:{}", mapped));
        if cap.get(1).is_some() {
            features.push("struct:field_pub".to_string());
        }
    }

    // Enums et variantes
    for cap in re_enum.captures_iter(code) {
        let name = &cap[2];
        features.push(format!("enum:{}", name));
        if cap.get(1).is_some() {
            features.push("enum:pub".to_string());
        }
        if let Some(generics) = cap.get(3) {
            features.push(format!("enum:generic:{}", generics.as_str()));
        }
    }
    for cap in re_enum_variant.captures_iter(code) {
        let variant = &cap[1];
        features.push(format!("enum:variant:{}", variant));
    }

    // Traits
    for cap in re_trait.captures_iter(code) {
        let name = &cap[2];
        features.push(format!("trait:{}", name));
        if cap.get(1).is_some() {
            features.push("trait:pub".to_string());
        }
        if let Some(generics) = cap.get(3) {
            features.push(format!("trait:generic:{}", generics.as_str()));
        }
    }

    // Impl blocks
    for cap in re_impl.captures_iter(code) {
        let target = &cap[2];
        features.push(format!("impl:{}", target));
    }

    // Macros
    for cap in re_macro.captures_iter(code) {
        features.push(format!("macro:{}", &cap[1]));
    }

    // Attributs
    for cap in re_attr.captures_iter(code) {
        features.push(format!("attr:{}", &cap[1]));
    }

    // Modules
    for cap in re_mod.captures_iter(code) {
        features.push(format!("mod:{}", &cap[1]));
    }

    // Imports
    for cap in re_use.captures_iter(code) {
        features.push(format!("use:{}", &cap[1]));
    }

    // Tests
    if re_test.is_match(code) {
        features.push("test".to_string());
    }

    features
}

pub fn extract_syn_features(code: &str) -> Vec<String> {
    let mut features = Vec::new();
    if let Ok(ast) = syn::parse_file(code) {
        struct FnVisitor<'a> {
            features: &'a mut Vec<String>,
        }
        impl<'ast> Visit<'ast> for FnVisitor<'_> {
            fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
                self.features.push(format!("fn:{}", node.sig.ident));
                // Type de retour (corrigé : pas de Debug sur ReturnType)
                match &node.sig.output {
                    syn::ReturnType::Default => {
                        self.features.push("fn:return:()".to_string());
                    }
                    syn::ReturnType::Type(_, ty) => {
                        let type_str = match &**ty {
                            syn::Type::Path(type_path) => type_path
                                .path
                                .segments
                                .last()
                                .map(|seg| seg.ident.to_string())
                                .unwrap_or_else(|| "unknown".to_string()),
                            _ => "unknown".to_string(),
                        };
                        // Utilisation effective du mapping sémantique
                        let mapped = crate::ai::ast::map_rust_type(&type_str);
                        self.features.push(format!("fn:return:{}", mapped));
                    }
                }
                // Visibilité
                if let syn::Visibility::Public(_) = node.vis {
                    self.features.push("fn:pub".to_string());
                }
                // Args
                for input in &node.sig.inputs {
                    if let syn::FnArg::Typed(arg) = input {
                        let type_str = match &*arg.ty {
                            syn::Type::Path(type_path) => type_path
                                .path
                                .segments
                                .last()
                                .map(|seg| seg.ident.to_string())
                                .unwrap_or_else(|| "unknown".to_string()),
                            _ => "unknown".to_string(),
                        };
                        // Utilisation effective du mapping sémantique
                        let mapped = crate::ai::ast::map_rust_type(&type_str);
                        self.features.push(format!("fn:arg:{}", mapped));
                    }
                }
            }
            fn visit_item_struct(&mut self, node: &'ast syn::ItemStruct) {
                self.features.push(format!("struct:{}", node.ident));
            }
            fn visit_item_enum(&mut self, node: &'ast syn::ItemEnum) {
                self.features.push(format!("enum:{}", node.ident));
            }
            // ...ajoute d'autres visites selon tes besoins...
        }
        let mut visitor = FnVisitor {
            features: &mut features,
        };
        visitor.visit_file(&ast);
    }
    features
}
