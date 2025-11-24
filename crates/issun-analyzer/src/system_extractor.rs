//! System structure extraction logic

use crate::types::SystemInfo;
use syn::{File, Item, ItemImpl, Type, TypePath};

/// Extract System implementations from a syntax tree
pub fn extract_systems(file_path: &str, syntax_tree: &File) -> Vec<SystemInfo> {
    let mut systems = Vec::new();

    for item in &syntax_tree.items {
        if let Item::Impl(item_impl) = item {
            if let Some(system_info) = analyze_system_impl(file_path, item_impl) {
                systems.push(system_info);
            }
        }
    }

    systems
}

/// Analyze an impl block to see if it implements System trait
fn analyze_system_impl(file_path: &str, item_impl: &ItemImpl) -> Option<SystemInfo> {
    // Check if this implements a trait
    let trait_path = item_impl.trait_.as_ref()?;

    // Get the trait name
    let trait_name = trait_path
        .1
        .segments
        .last()
        .map(|seg| seg.ident.to_string())?;

    // Only interested in System trait
    if trait_name != "System" {
        return None;
    }

    // Get the struct name being implemented
    let struct_name = match &*item_impl.self_ty {
        Type::Path(TypePath { path, .. }) => path.segments.last()?.ident.to_string(),
        _ => return None,
    };

    // Extract module path from file path
    let module_path = file_path_to_module_path(file_path);

    // TODO: Extract subscribes, publishes, hooks, states from the impl methods
    // For now, return basic info
    Some(SystemInfo {
        name: struct_name,
        module_path,
        file_path: file_path.to_string(),
        subscribes: Vec::new(),
        publishes: Vec::new(),
        hooks: Vec::new(),
        states: Vec::new(),
    })
}

/// Convert file path to module path
/// e.g., "crates/issun/src/plugin/combat/system.rs" -> "plugin::combat::system"
fn file_path_to_module_path(file_path: &str) -> String {
    // Remove "crates/issun/src/" prefix
    let path = file_path
        .trim_start_matches("crates/issun/src/")
        .trim_end_matches(".rs");

    // Replace "/" with "::"
    path.replace('/', "::")
}

/// Extract System struct fields to find hooks, states, configs
pub fn extract_system_fields(syntax_tree: &File, system_name: &str) -> SystemFieldInfo {
    let mut info = SystemFieldInfo {
        hooks: Vec::new(),
        states: Vec::new(),
        configs: Vec::new(),
    };

    for item in &syntax_tree.items {
        if let Item::Struct(item_struct) = item {
            if item_struct.ident == system_name {
                // Analyze fields
                for field in &item_struct.fields {
                    if let Some(field_name) = &field.ident {
                        let field_name_str = field_name.to_string();
                        let type_str = type_to_string(&field.ty);

                        // Detect hooks: Arc<dyn XxxHook>
                        if type_str.contains("Hook") && type_str.contains("Arc") {
                            info.hooks.push(extract_hook_name(&type_str));
                        }
                        // Detect states/configs by field name
                        else if field_name_str.contains("state") {
                            info.states.push(type_str);
                        } else if field_name_str.contains("config") {
                            info.configs.push(type_str);
                        }
                    }
                }
            }
        }
    }

    info
}

/// Information about System struct fields
#[derive(Debug, Clone)]
pub struct SystemFieldInfo {
    pub hooks: Vec<String>,
    pub states: Vec<String>,
    pub configs: Vec<String>,
}

/// Convert Type to String (full representation with generics)
fn type_to_string(ty: &Type) -> String {
    use syn::{GenericArgument, PathArguments};

    match ty {
        Type::Path(type_path) => {
            let segments: Vec<String> = type_path
                .path
                .segments
                .iter()
                .map(|seg| {
                    let ident = seg.ident.to_string();
                    match &seg.arguments {
                        PathArguments::AngleBracketed(args) => {
                            let inner: Vec<String> = args
                                .args
                                .iter()
                                .map(|arg| match arg {
                                    GenericArgument::Type(inner_ty) => type_to_string(inner_ty),
                                    _ => "".to_string(),
                                })
                                .filter(|s| !s.is_empty())
                                .collect();
                            if inner.is_empty() {
                                ident
                            } else {
                                format!("{}<{}>", ident, inner.join(", "))
                            }
                        }
                        _ => ident,
                    }
                })
                .collect();
            segments.join("::")
        }
        Type::TraitObject(trait_obj) => {
            // Handle dyn Trait
            let bounds: Vec<String> = trait_obj
                .bounds
                .iter()
                .filter_map(|bound| {
                    if let syn::TypeParamBound::Trait(trait_bound) = bound {
                        Some(
                            trait_bound
                                .path
                                .segments
                                .iter()
                                .map(|seg| seg.ident.to_string())
                                .collect::<Vec<_>>()
                                .join("::"),
                        )
                    } else {
                        None
                    }
                })
                .collect();
            format!("dyn {}", bounds.join(" + "))
        }
        _ => "Unknown".to_string(),
    }
}

/// Extract hook name from Arc<dyn XxxHook> pattern
fn extract_hook_name(type_str: &str) -> String {
    // Pattern: Arc<dyn CombatHook> -> CombatHook
    // Pattern: Arc<CombatHook> -> CombatHook (also valid)

    // Remove "Arc<" and ">"
    let inner = type_str
        .trim_start_matches("Arc<")
        .trim_end_matches('>')
        .trim();

    // Remove "dyn "
    let hook_name = inner.trim_start_matches("dyn ").trim();

    hook_name.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_system_impl() {
        let code = r#"
            use crate::system::System;

            #[async_trait]
            impl System for CombatSystem {
                async fn update(&mut self, ctx: &mut Context) {
                    // implementation
                }
            }
        "#;

        let syntax_tree = syn::parse_file(code).unwrap();
        let systems = extract_systems("crates/issun/src/plugin/combat/system.rs", &syntax_tree);

        assert_eq!(systems.len(), 1);
        assert_eq!(systems[0].name, "CombatSystem");
        assert_eq!(systems[0].module_path, "plugin::combat::system");
    }

    #[test]
    fn test_extract_system_fields() {
        let code = r#"
            pub struct CombatSystem {
                hook: Arc<dyn CombatHook>,
                state: CombatState,
            }
        "#;

        let syntax_tree = syn::parse_file(code).unwrap();
        let field_info = extract_system_fields(&syntax_tree, "CombatSystem");

        assert_eq!(field_info.hooks.len(), 1);
        assert!(field_info.hooks[0].contains("CombatHook"));
    }

    #[test]
    fn test_file_path_to_module_path() {
        assert_eq!(
            file_path_to_module_path("crates/issun/src/plugin/combat/system.rs"),
            "plugin::combat::system"
        );
    }
}
