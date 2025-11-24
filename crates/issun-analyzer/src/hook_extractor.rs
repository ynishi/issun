//! Hook trait analysis and call site detection

use crate::types::{HookCall, HookCategory, HookInfo, HookMethod};
use syn::{
    visit::Visit, File, FnArg, ImplItemFn, Item, ItemTrait, ReturnType, TraitItem, TraitItemFn,
    Type,
};

/// Extract all Hook trait definitions from a file
pub fn extract_hook_traits(file_path: &str, syntax_tree: &File) -> Vec<HookInfo> {
    let mut hooks = Vec::new();

    for item in &syntax_tree.items {
        if let Item::Trait(item_trait) = item {
            if is_hook_trait(item_trait) {
                let hook_info = analyze_hook_trait(file_path, item_trait);
                hooks.push(hook_info);
            }
        }
    }

    hooks
}

/// Check if a trait is a Hook trait (ends with "Hook")
fn is_hook_trait(item_trait: &ItemTrait) -> bool {
    item_trait.ident.to_string().ends_with("Hook")
}

/// Analyze a Hook trait and extract all methods
fn analyze_hook_trait(file_path: &str, item_trait: &ItemTrait) -> HookInfo {
    let trait_name = item_trait.ident.to_string();
    let module_path = file_path_to_module_path(file_path);

    let mut methods = Vec::new();

    for trait_item in &item_trait.items {
        if let TraitItem::Fn(trait_fn) = trait_item {
            let method = analyze_hook_method(trait_fn);
            methods.push(method);
        }
    }

    HookInfo {
        trait_name,
        module_path,
        file_path: file_path.to_string(),
        methods,
    }
}

/// Analyze a single Hook method
fn analyze_hook_method(trait_fn: &TraitItemFn) -> HookMethod {
    let name = trait_fn.sig.ident.to_string();
    let category = categorize_hook_method(&name);

    // Extract parameter types
    let params: Vec<String> = trait_fn
        .sig
        .inputs
        .iter()
        .filter_map(|arg| match arg {
            FnArg::Receiver(_) => None, // Skip self
            FnArg::Typed(pat_type) => Some(type_to_string(&pat_type.ty)),
        })
        .collect();

    // Extract return type
    let return_type = match &trait_fn.sig.output {
        ReturnType::Default => "()".to_string(),
        ReturnType::Type(_, ty) => type_to_string(ty),
    };

    // Check if has default implementation
    let has_default_impl = trait_fn.default.is_some();

    HookMethod {
        name,
        category,
        params,
        return_type,
        has_default_impl,
    }
}

/// Categorize a hook method based on its name
fn categorize_hook_method(name: &str) -> HookCategory {
    if name.starts_with("on_") {
        HookCategory::Notification
    } else if name.starts_with("can_") {
        HookCategory::Validation
    } else if name.starts_with("before_") || name.starts_with("after_") {
        HookCategory::Lifecycle
    } else if name.starts_with("calculate_") {
        HookCategory::Calculation
    } else if name.starts_with("generate_") {
        HookCategory::Generation
    } else {
        HookCategory::Other
    }
}

/// Extract Hook method calls from implementation code
pub fn extract_hook_calls(file_path: &str, syntax_tree: &File) -> Vec<HookCall> {
    let mut visitor = HookCallVisitor {
        file_path: file_path.to_string(),
        calls: Vec::new(),
        current_function: None,
    };

    visitor.visit_file(syntax_tree);
    visitor.calls
}

/// Visitor for finding Hook method calls
struct HookCallVisitor {
    file_path: String,
    calls: Vec<HookCall>,
    current_function: Option<String>,
}

impl<'ast> Visit<'ast> for HookCallVisitor {
    fn visit_impl_item_fn(&mut self, node: &'ast ImplItemFn) {
        let old_function = self.current_function.clone();
        self.current_function = Some(node.sig.ident.to_string());

        // Visit function body
        syn::visit::visit_impl_item_fn(self, node);

        self.current_function = old_function;
    }

    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        let old_function = self.current_function.clone();
        self.current_function = Some(node.sig.ident.to_string());

        // Visit function body
        syn::visit::visit_item_fn(self, node);

        self.current_function = old_function;
    }

    fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
        use syn::Expr;

        // Check if this looks like a hook call: self.hook.method_name()
        if let Expr::Field(field_expr) = &*node.receiver {
            let field_name = &field_expr.member;
            let field_str = format!("{}", quote::quote! { #field_name });

            // Check if field name contains "hook"
            if field_str.contains("hook") {
                let method_name = node.method.to_string();

                // Record this as a potential hook call
                self.calls.push(HookCall {
                    hook_trait: field_str, // We'll infer the trait name later
                    method_name,
                    caller: self
                        .current_function
                        .clone()
                        .unwrap_or_else(|| "unknown".to_string()),
                    file_path: self.file_path.clone(),
                    line: 0,
                });
            }
        }

        // Continue visiting
        syn::visit::visit_expr_method_call(self, node);
    }
}

/// Convert file path to module path
fn file_path_to_module_path(file_path: &str) -> String {
    file_path
        .trim_start_matches("crates/issun/src/")
        .trim_end_matches(".rs")
        .replace('/', "::")
}

/// Convert Type to String (simplified)
fn type_to_string(ty: &Type) -> String {
    format!("{}", quote::quote! { #ty })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_categorize_hook_method() {
        assert_eq!(
            categorize_hook_method("on_travel_started"),
            HookCategory::Notification
        );
        assert_eq!(
            categorize_hook_method("can_start_travel"),
            HookCategory::Validation
        );
        assert_eq!(
            categorize_hook_method("before_turn"),
            HookCategory::Lifecycle
        );
        assert_eq!(
            categorize_hook_method("calculate_speed"),
            HookCategory::Calculation
        );
        assert_eq!(
            categorize_hook_method("generate_encounter"),
            HookCategory::Generation
        );
        assert_eq!(categorize_hook_method("custom_method"), HookCategory::Other);
    }

    #[test]
    fn test_extract_hook_traits() {
        let code = r#"
            #[async_trait]
            pub trait CombatHook: Send + Sync {
                async fn on_battle_start(&self, battle_id: &str) {}
                async fn can_attack(&self, attacker: &str) -> Result<(), String> {
                    Ok(())
                }
                async fn calculate_damage(&self, base: u32) -> u32 {
                    base
                }
            }
        "#;

        let syntax_tree = syn::parse_file(code).unwrap();
        let hooks = extract_hook_traits("test.rs", &syntax_tree);

        assert_eq!(hooks.len(), 1);
        assert_eq!(hooks[0].trait_name, "CombatHook");
        assert_eq!(hooks[0].methods.len(), 3);

        assert_eq!(hooks[0].methods[0].name, "on_battle_start");
        assert_eq!(hooks[0].methods[0].category, HookCategory::Notification);

        assert_eq!(hooks[0].methods[1].name, "can_attack");
        assert_eq!(hooks[0].methods[1].category, HookCategory::Validation);

        assert_eq!(hooks[0].methods[2].name, "calculate_damage");
        assert_eq!(hooks[0].methods[2].category, HookCategory::Calculation);
    }
}
