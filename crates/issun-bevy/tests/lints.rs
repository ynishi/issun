// cSpell:ignore issun walkdir
//! Linting Tests for issun-bevy Best Practices
//!
//! Enforces coding standards via static analysis:
//! 1. Reflect attributes on Bevy types
//! 2. Entity query safety (no .unwrap() on queries)
//! 3. Config resource Default implementation
//!
//! ⚠️ IMPORTANT: Bevy 0.17 Message types only require #[derive(Reflect)]
//! - ReflectMessage helper does NOT exist in Bevy 0.17
//! - Message trait is just: `Send + Sync + 'static`
//! - #[reflect(Message)] is not needed (and will cause compile errors)

use std::collections::HashSet;
use std::fs;
use std::path::Path;
use syn::{visit::Visit, Expr, ExprMethodCall, Item, ItemFn, ItemStruct};
use walkdir::WalkDir;

struct ReflectVisitor {
    errors: Vec<String>,
    current_file: String,
}

/// Check a directory for Reflect violations
fn check_reflect_violations(target_dir: &str) -> Vec<String> {
    let mut visitor = ReflectVisitor {
        errors: Vec::new(),
        current_file: String::new(),
    };

    if !Path::new(target_dir).exists() {
        return Vec::new();
    }

    for entry in WalkDir::new(target_dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "rs") {
            visitor.current_file = path.display().to_string();
            let content = fs::read_to_string(path).unwrap();
            if let Ok(file) = syn::parse_file(&content) {
                visitor.visit_file(&file);
            }
        }
    }

    visitor.errors
}

impl<'ast> Visit<'ast> for ReflectVisitor {
    fn visit_item_struct(&mut self, node: &'ast ItemStruct) {
        // Detect structs that derive Component/Resource/Message/Event
        // ⚠️ Bevy 0.17: buffered events use Message, observer events use Event
        let derived_types = ["Component", "Resource", "Message", "Event"];

        for ty in &derived_types {
            if self.has_derive(node, ty) {
                if !self.has_derive(node, "Reflect") {
                    self.errors.push(format!(
                        "{} - '{}' derives {} but missing #[derive(Reflect)]",
                        self.current_file, node.ident, ty
                    ));
                }

                // ⚠️ CRITICAL: Bevy 0.17 doesn't have ReflectMessage or ReflectEvent
                // Message and Event types only need #[derive(Reflect)], not #[reflect(...)]
                // See: https://github.com/bevyengine/bevy/discussions/11587
                if *ty != "Message" && *ty != "Event" && !self.has_reflect_attr(node, ty) {
                    self.errors.push(format!(
                        "{} - '{}' derives {} but missing #[reflect({})]",
                        self.current_file, node.ident, ty, ty
                    ));
                }
            }
        }

        syn::visit::visit_item_struct(self, node);
    }
}

impl ReflectVisitor {
    fn has_derive(&self, node: &ItemStruct, name: &str) -> bool {
        node.attrs.iter().any(|attr| {
            if !attr.path().is_ident("derive") {
                return false;
            }
            attr.parse_args_with(
                syn::punctuated::Punctuated::<syn::Path, syn::Token![,]>::parse_terminated,
            )
            .map(|list| list.iter().any(|p| p.is_ident(name)))
            .unwrap_or(false)
        })
    }

    fn has_reflect_attr(&self, node: &ItemStruct, ty: &str) -> bool {
        use quote::ToTokens;
        node.attrs.iter().any(|attr| {
            attr.path().is_ident("reflect") && attr.meta.to_token_stream().to_string().contains(ty)
        })
    }
}

//
// ═══════════════════════════════════════════════════════════════════════════
// Entity Query Safety Lint
// ═══════════════════════════════════════════════════════════════════════════
//

struct QuerySafetyVisitor {
    errors: Vec<String>,
    current_file: String,
    in_test_code: bool,
}

/// Check for unsafe .unwrap() usage on Query::get() calls
fn check_query_safety_violations(target_dir: &str) -> Vec<String> {
    let mut visitor = QuerySafetyVisitor {
        errors: Vec::new(),
        current_file: String::new(),
        in_test_code: false,
    };

    if !Path::new(target_dir).exists() {
        return Vec::new();
    }

    for entry in WalkDir::new(target_dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "rs") {
            visitor.current_file = path.display().to_string();
            let content = fs::read_to_string(path).unwrap();
            if let Ok(file) = syn::parse_file(&content) {
                visitor.visit_file(&file);
            }
        }
    }

    visitor.errors
}

impl<'ast> Visit<'ast> for QuerySafetyVisitor {
    fn visit_item_fn(&mut self, node: &'ast ItemFn) {
        // Check if this is a test function
        let was_in_test = self.in_test_code;
        let is_test = node
            .attrs
            .iter()
            .any(|attr| attr.path().is_ident("test") || attr.path().is_ident("cfg"));

        if is_test {
            self.in_test_code = true;
        }

        syn::visit::visit_item_fn(self, node);

        self.in_test_code = was_in_test;
    }

    fn visit_expr_method_call(&mut self, node: &'ast ExprMethodCall) {
        // Skip if we're in test code
        if !self.in_test_code {
            // Check for .unwrap() after .get() or .get_mut()
            if node.method == "unwrap" {
                if let Expr::MethodCall(inner) = &*node.receiver {
                    if inner.method == "get" || inner.method == "get_mut" {
                        // Check if the receiver looks like a query
                        // This is a heuristic: if the previous method call is on something
                        // that could be a Query, flag it
                        self.errors.push(format!(
                            "{} - Unsafe .unwrap() on query.{}(). Use 'if let Ok(...) = query.{}(...)' instead",
                            self.current_file,
                            inner.method,
                            inner.method
                        ));
                    }
                }
            }

            // Check for .expect() after .get() or .get_mut()
            if node.method == "expect" {
                if let Expr::MethodCall(inner) = &*node.receiver {
                    if inner.method == "get" || inner.method == "get_mut" {
                        self.errors.push(format!(
                            "{} - Unsafe .expect() on query.{}(). Use 'if let Ok(...) = query.{}(...)' instead",
                            self.current_file,
                            inner.method,
                            inner.method
                        ));
                    }
                }
            }
        }

        syn::visit::visit_expr_method_call(self, node);
    }
}

//
// ═══════════════════════════════════════════════════════════════════════════
// Config Resource Default Lint
// ═══════════════════════════════════════════════════════════════════════════
//

/// Check that Config resources have Default implementation
fn check_config_default_violations(target_dir: &str) -> Vec<String> {
    let mut errors = Vec::new();

    if !Path::new(target_dir).exists() {
        return Vec::new();
    }

    for entry in WalkDir::new(target_dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "rs") {
            let file_path = path.display().to_string();
            let content = fs::read_to_string(path).unwrap();
            if let Ok(file) = syn::parse_file(&content) {
                // Find Config structs
                let config_structs = file
                    .items
                    .iter()
                    .filter_map(|item| {
                        if let Item::Struct(s) = item {
                            let is_config = s.ident.to_string().ends_with("Config");
                            let has_resource = s.attrs.iter().any(|attr| {
                                if attr.path().is_ident("derive") {
                                    attr
                                            .parse_args_with(
                                                syn::punctuated::Punctuated::<
                                                    syn::Path,
                                                    syn::Token![,],
                                                >::parse_terminated,
                                            )
                                            .map(|list| list.iter().any(|p| p.is_ident("Resource")))
                                            .unwrap_or(false)
                                } else {
                                    false
                                }
                            });

                            if is_config && has_resource {
                                Some(s.ident.to_string())
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>();

                // Find Default implementations
                // cSpell:ignore impls
                let default_impls = file
                    .items
                    .iter()
                    .filter_map(|item| {
                        if let Item::Impl(imp) = item {
                            if let Some((_, trait_path, _)) = &imp.trait_ {
                                if trait_path
                                    .segments
                                    .last()
                                    .map(|s| s.ident == "Default")
                                    .unwrap_or(false)
                                {
                                    if let syn::Type::Path(type_path) = &*imp.self_ty {
                                        return type_path
                                            .path
                                            .segments
                                            .last()
                                            .map(|s| s.ident.to_string());
                                    }
                                }
                            }
                        }
                        None
                    })
                    .collect::<HashSet<_>>();

                // Check each Config struct has Default
                for config in config_structs {
                    if !default_impls.contains(&config) {
                        errors.push(format!(
                            "{} - '{}' is a Resource Config but missing 'impl Default'",
                            file_path, config
                        ));
                    }
                }
            }
        }
    }

    errors
}

//
// ═══════════════════════════════════════════════════════════════════════════
// System Ordering Lint
// ═══════════════════════════════════════════════════════════════════════════
//

use syn::{FnArg, ImplItem, ItemImpl, Type};

struct SystemOrderingVisitor {
    errors: Vec<String>,
    current_file: String,
}

fn check_system_ordering_violations(target_dir: &str) -> Vec<String> {
    let mut visitor = SystemOrderingVisitor {
        errors: Vec::new(),
        current_file: String::new(),
    };

    if !Path::new(target_dir).exists() {
        return Vec::new();
    }

    for entry in WalkDir::new(target_dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "rs")
            && path.file_name().is_some_and(|name| name == "plugin.rs")
        {
            visitor.current_file = path.display().to_string();
            let content = fs::read_to_string(path).unwrap();
            if let Ok(file) = syn::parse_file(&content) {
                visitor.visit_file(&file);
            }
        }
    }

    visitor.errors
}

impl<'ast> Visit<'ast> for SystemOrderingVisitor {
    fn visit_item_impl(&mut self, node: &'ast ItemImpl) {
        if let Type::Path(type_path) = &*node.self_ty {
            let type_name = type_path
                .path
                .segments
                .last()
                .map(|s| s.ident.to_string())
                .unwrap_or_default();

            if type_name.ends_with("Plugin") {
                for item in &node.items {
                    if let ImplItem::Fn(method) = item {
                        if method.sig.ident == "build" {
                            let body_str = quote::quote!(#method).to_string();

                            if body_str.contains("add_systems") && !body_str.contains("in_set") {
                                self.errors.push(format!(
                                    "{} - Plugin::build() calls add_systems without .in_set(). \
                                    Systems should use IssunSet for deterministic ordering.",
                                    self.current_file
                                ));
                            }
                        }
                    }
                }
            }
        }

        syn::visit::visit_item_impl(self, node);
    }
}

//
// ═══════════════════════════════════════════════════════════════════════════
// System Parameter Validation Lint
// ═══════════════════════════════════════════════════════════════════════════
//

struct SystemParamVisitor {
    errors: Vec<String>,
    current_file: String,
}

fn check_system_param_violations(target_dir: &str) -> Vec<String> {
    let mut visitor = SystemParamVisitor {
        errors: Vec::new(),
        current_file: String::new(),
    };

    if !Path::new(target_dir).exists() {
        return Vec::new();
    }

    for entry in WalkDir::new(target_dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "rs")
            && path.file_name().is_some_and(|name| name == "systems.rs")
        {
            visitor.current_file = path.display().to_string();
            let content = fs::read_to_string(path).unwrap();
            if let Ok(file) = syn::parse_file(&content) {
                visitor.visit_file(&file);
            }
        }
    }

    visitor.errors
}

impl<'ast> Visit<'ast> for SystemParamVisitor {
    fn visit_item_fn(&mut self, node: &'ast ItemFn) {
        let mut has_world_ref = false;
        let mut has_mut_query = false;

        for input in &node.sig.inputs {
            if let FnArg::Typed(pat_type) = input {
                let type_str = quote::quote!(#pat_type.ty).to_string();

                if type_str.contains("& World") {
                    has_world_ref = true;
                }

                if type_str.contains("Query") && type_str.contains("& mut") {
                    has_mut_query = true;
                }
            }
        }

        if has_world_ref && has_mut_query {
            self.errors.push(format!(
                "{} - Function '{}' uses both &World and Query<&mut ...>. \
                This causes borrowing conflicts. Use Query results for validation instead.",
                self.current_file, node.sig.ident
            ));
        }

        syn::visit::visit_item_fn(self, node);
    }
}

//
// ═══════════════════════════════════════════════════════════════════════════
// Entity::from_bits Safety Lint (P0)
// ═══════════════════════════════════════════════════════════════════════════
//

struct EntityFromBitsSafetyVisitor {
    errors: Vec<String>,
    current_file: String,
    in_test_code: bool,
}

fn check_entity_from_bits_violations(target_dir: &str) -> Vec<String> {
    let mut visitor = EntityFromBitsSafetyVisitor {
        errors: Vec::new(),
        current_file: String::new(),
        in_test_code: false,
    };

    if !Path::new(target_dir).exists() {
        return Vec::new();
    }

    for entry in WalkDir::new(target_dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "rs") {
            // Skip entity_safety.rs - that's where the safe wrappers are defined
            if path.file_name().is_some_and(|name| name == "entity_safety.rs") {
                continue;
            }

            visitor.current_file = path.display().to_string();
            let content = fs::read_to_string(path).unwrap();
            if let Ok(file) = syn::parse_file(&content) {
                visitor.visit_file(&file);
            }
        }
    }

    visitor.errors
}

impl<'ast> Visit<'ast> for EntityFromBitsSafetyVisitor {
    fn visit_item_fn(&mut self, node: &'ast ItemFn) {
        // Check if this is a test function
        let was_in_test = self.in_test_code;
        let is_test = node
            .attrs
            .iter()
            .any(|attr| attr.path().is_ident("test") || attr.path().is_ident("cfg"));

        if is_test {
            self.in_test_code = true;
        }

        syn::visit::visit_item_fn(self, node);

        self.in_test_code = was_in_test;
    }

    fn visit_expr_call(&mut self, node: &'ast syn::ExprCall) {
        use quote::ToTokens;

        // Skip if we're in test code
        if !self.in_test_code {
            // Check for Entity::from_bits() calls
            // This is an associated function call like Entity::from_bits(12345)
            let func_str = node.func.to_token_stream().to_string();

            if func_str.contains("Entity") && func_str.contains("from_bits") {
                // ⚠️ P0 VIOLATION: Entity::from_bits() must be wrapped in SafeEntityRef
                // The only safe usage is: SafeEntityRef::new(Entity::from_bits(...), world)
                // or via entity_from_bits_safe() helper
                self.errors.push(format!(
                    "{} - Unsafe Entity::from_bits() call. Use entity_from_bits_safe() or SafeEntityRef::new() instead. \
                    ⚠️ CRITICAL P0: Entity::from_bits() creates entities that may be despawned, causing crashes!",
                    self.current_file
                ));
            }
        }

        syn::visit::visit_expr_call(self, node);
    }
}

//
// ═══════════════════════════════════════════════════════════════════════════
// Main Lint Tests
// ═══════════════════════════════════════════════════════════════════════════
//

#[test]
fn enforce_reflect_on_all_bevy_types() {
    let errors = check_reflect_violations("src/plugins");

    if errors.is_empty() && !Path::new("src/plugins").exists() {
        eprintln!("⚠️  Warning: src/plugins not found, skipping Reflect lint check");
        return;
    }

    assert!(
        errors.is_empty(),
        "\n\n❌ Reflect Lint Errors:\n\n{}\n\n\
        💡 Fix: Add #[derive(Reflect)] and #[reflect(Component/Resource/Message/Event)]\n",
        errors.join("\n")
    );
}

#[test]
fn enforce_query_safety() {
    let errors = check_query_safety_violations("src/plugins");

    if errors.is_empty() && !Path::new("src/plugins").exists() {
        eprintln!("⚠️  Warning: src/plugins not found, skipping Query Safety lint check");
        return;
    }

    // cSpell:ignore despawned
    assert!(
        errors.is_empty(),
        "\n\n❌ Query Safety Lint Errors:\n\n{}\n\n\
        💡 Fix: Use 'if let Ok(x) = query.get(entity)' instead of 'query.get(entity).unwrap()'\n\
        ⚠️  CRITICAL: .unwrap() on queries causes panics when entities are despawned!\n",
        errors.join("\n")
    );
}

#[test]
fn enforce_config_default() {
    let errors = check_config_default_violations("src/plugins");

    if errors.is_empty() && !Path::new("src/plugins").exists() {
        eprintln!("⚠️  Warning: src/plugins not found, skipping Config Default lint check");
        return;
    }

    assert!(
        errors.is_empty(),
        "\n\n❌ Config Default Lint Errors:\n\n{}\n\n\
        💡 Fix: Add 'impl Default for YourConfig {{ ... }}'\n",
        errors.join("\n")
    );
}

#[test]
fn enforce_system_ordering() {
    let errors = check_system_ordering_violations("src/plugins");

    if errors.is_empty() && !Path::new("src/plugins").exists() {
        eprintln!("⚠️  Warning: src/plugins not found, skipping System Ordering lint check");
        return;
    }

    assert!(
        errors.is_empty(),
        "\n\n❌ System Ordering Lint Errors:\n\n{}\n\n\
        💡 Fix: Use .in_set(IssunSet::Logic) or appropriate set for deterministic ordering\n\
        Example: app.add_systems(Update, my_system.in_set(IssunSet::Logic));\n",
        errors.join("\n")
    );
}

#[test]
fn enforce_system_params() {
    let errors = check_system_param_violations("src/plugins");

    if errors.is_empty() && !Path::new("src/plugins").exists() {
        eprintln!("⚠️  Warning: src/plugins not found, skipping System Params lint check");
        return;
    }

    assert!(
        errors.is_empty(),
        "\n\n❌ System Parameter Lint Errors:\n\n{}\n\n\
        💡 Fix: Remove &World parameter and use Query results for entity validation\n\
        Example: if let Ok(component) = query.get(entity) {{ /* safe */ }}\n",
        errors.join("\n")
    );
}

#[test]
fn enforce_entity_from_bits_safety() {
    let errors = check_entity_from_bits_violations("src/plugins");

    if errors.is_empty() && !Path::new("src/plugins").exists() {
        eprintln!("⚠️  Warning: src/plugins not found, skipping Entity::from_bits Safety lint check");
        return;
    }

    assert!(
        errors.is_empty(),
        "\n\n❌ Entity::from_bits Safety Lint Errors (P0):\n\n{}\n\n\
        💡 Fix: Use entity_from_bits_safe() or SafeEntityRef::new() wrapper\n\
        Example: let safe_entity = entity_from_bits_safe(bits, &world);\n\
        ⚠️  CRITICAL P0: Entity::from_bits() without safety checks causes crashes when entities are despawned!\n",
        errors.join("\n")
    );
}

#[cfg(test)]
mod linter_tests {
    use super::*;
    use std::fs;
    use std::io::Write;

    /// Test that the linter correctly detects missing #[derive(Reflect)]
    #[test]
    fn test_linter_detects_missing_derive_reflect() {
        let test_dir = "tests/lints_fixtures/missing_derive";
        fs::create_dir_all(test_dir).unwrap();

        let test_file = format!("{}/test.rs", test_dir);
        let mut file = fs::File::create(&test_file).unwrap();
        writeln!(
            file,
            r#"
use bevy::prelude::*;

#[derive(Component)]
pub struct BadComponent {{
    pub value: i32,
}}
"#
        )
        .unwrap();

        let errors = check_reflect_violations(test_dir);

        // Cleanup
        fs::remove_dir_all(test_dir).unwrap();

        // Assert: Should detect 2 errors
        assert_eq!(
            errors.len(),
            2,
            "Expected 2 errors, got {}: {:?}",
            errors.len(),
            errors
        );
        assert!(
            errors
                .iter()
                .any(|e| e.contains("BadComponent") && e.contains("missing #[derive(Reflect)]")),
            "Expected error about missing #[derive(Reflect)], got: {:?}",
            errors
        );
        assert!(
            errors
                .iter()
                .any(|e| e.contains("BadComponent") && e.contains("missing #[reflect(Component)]")),
            "Expected error about missing #[reflect(Component)], got: {:?}",
            errors
        );
    }

    /// Test that the linter correctly detects missing #[reflect(...)]
    #[test]
    fn test_linter_detects_missing_reflect_attribute() {
        let test_dir = "tests/lints_fixtures/missing_attribute";
        fs::create_dir_all(test_dir).unwrap();

        let test_file = format!("{}/test.rs", test_dir);
        let mut file = fs::File::create(&test_file).unwrap();
        writeln!(
            file,
            r#"
use bevy::prelude::*;

#[derive(Component, Reflect)]
pub struct BadComponent {{
    pub value: i32,
}}
"#
        )
        .unwrap();

        let errors = check_reflect_violations(test_dir);

        // Cleanup
        fs::remove_dir_all(test_dir).unwrap();

        // Assert: Only error for missing #[reflect(Component)]
        assert_eq!(
            errors.len(),
            1,
            "Expected 1 error, got {}: {:?}",
            errors.len(),
            errors
        );
        assert!(
            errors[0].contains("BadComponent")
                && errors[0].contains("missing #[reflect(Component)]"),
            "Expected error about missing #[reflect(Component)], got: {}",
            errors[0]
        );
    }

    /// Test that the linter accepts correct usage
    #[test]
    fn test_linter_accepts_correct_usage() {
        let test_dir = "tests/lints_fixtures/correct";
        fs::create_dir_all(test_dir).unwrap();

        let test_file = format!("{}/test.rs", test_dir);
        let mut file = fs::File::create(&test_file).unwrap();
        writeln!(
            file,
            r#"
use bevy::prelude::*;

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct GoodComponent {{
    pub value: i32,
}}

#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct GoodResource {{
    pub count: u32,
}}

#[derive(Message, Clone, Reflect)]
#[reflect(Message)]
pub struct GoodMessage {{
    pub data: String,
}}
"#
        )
        .unwrap();

        let errors = check_reflect_violations(test_dir);

        // Cleanup
        fs::remove_dir_all(test_dir).unwrap();

        // Assert: No errors
        assert!(
            errors.is_empty(),
            "Expected no errors for correct usage, got: {:?}",
            errors
        );
    }

    /// Test that the linter detects violations for all types
    #[test]
    fn test_linter_detects_all_bevy_types() {
        let test_dir = "tests/lints_fixtures/all_types";
        fs::create_dir_all(test_dir).unwrap();

        let test_file = format!("{}/test.rs", test_dir);
        let mut file = fs::File::create(&test_file).unwrap();
        writeln!(
            file,
            r#"
use bevy::prelude::*;

#[derive(Component)]
pub struct BadComponent {{ pub v: i32 }}

#[derive(Resource)]
pub struct BadResource {{ pub v: i32 }}

#[derive(Message, Clone)]
pub struct BadMessage {{ pub v: i32 }}

#[derive(Event)]
pub struct BadEvent {{ pub v: i32 }}
"#
        )
        .unwrap();

        let errors = check_reflect_violations(test_dir);

        // Cleanup
        fs::remove_dir_all(test_dir).unwrap();

        // Assert: 6 errors total
        // - Component: 2 errors (#[derive(Reflect)] + #[reflect(Component)])
        // - Resource: 2 errors (#[derive(Reflect)] + #[reflect(Resource)])
        // - Message: 1 error (#[derive(Reflect)] only, no #[reflect(Message)])
        // - Event: 1 error (#[derive(Reflect)] only, no #[reflect(Event)])
        assert_eq!(
            errors.len(),
            6,
            "Expected 6 errors, got {}: {:?}",
            errors.len(),
            errors
        );

        // Component
        assert!(errors
            .iter()
            .any(|e| e.contains("BadComponent") && e.contains("missing #[derive(Reflect)]")));
        assert!(errors
            .iter()
            .any(|e| e.contains("BadComponent") && e.contains("missing #[reflect(Component)]")));

        // Resource
        assert!(errors
            .iter()
            .any(|e| e.contains("BadResource") && e.contains("missing #[derive(Reflect)]")));
        assert!(errors
            .iter()
            .any(|e| e.contains("BadResource") && e.contains("missing #[reflect(Resource)]")));

        // Message (⚠️ Only #[derive(Reflect)] required, no #[reflect(Message)])
        assert!(errors
            .iter()
            .any(|e| e.contains("BadMessage") && e.contains("missing #[derive(Reflect)]")));
        // ❌ No longer checking for #[reflect(Message)] - it doesn't exist in Bevy 0.17

        // Event (⚠️ Only #[derive(Reflect)] required, no #[reflect(Event)])
        assert!(errors
            .iter()
            .any(|e| e.contains("BadEvent") && e.contains("missing #[derive(Reflect)]")));
        // ❌ No longer checking for #[reflect(Event)] - it doesn't exist in Bevy 0.17
    }

    /// Test Query Safety: Detects .unwrap() on query.get()
    #[test]
    fn test_query_safety_detects_unwrap() {
        let test_dir = "tests/lints_fixtures/query_unsafe";
        fs::create_dir_all(test_dir).unwrap();

        let test_file = format!("{}/test.rs", test_dir);
        let mut file = fs::File::create(&test_file).unwrap();
        writeln!(
            file,
            r#"
use bevy::prelude::*;

pub fn bad_system(query: Query<&Health>) {{
    let health = query.get(entity).unwrap();  // ❌ Should error
}}
"#
        )
        .unwrap();

        let errors = check_query_safety_violations(test_dir);

        // Cleanup
        fs::remove_dir_all(test_dir).unwrap();

        // Assert: Should detect 1 error
        assert!(!errors.is_empty(), "Expected errors, got none");
        assert!(
            errors[0].contains("Unsafe .unwrap() on query.get()"),
            "Expected error about unsafe unwrap, got: {}",
            errors[0]
        );
    }

    /// Test Query Safety: Accepts if-let pattern
    #[test]
    fn test_query_safety_accepts_if_let() {
        let test_dir = "tests/lints_fixtures/query_safe";
        fs::create_dir_all(test_dir).unwrap();

        let test_file = format!("{}/test.rs", test_dir);
        let mut file = fs::File::create(&test_file).unwrap();
        writeln!(
            file,
            r#"
use bevy::prelude::*;

pub fn good_system(query: Query<&Health>) {{
    if let Ok(health) = query.get(entity) {{
        // ✅ Safe pattern
    }}
}}
"#
        )
        .unwrap();

        let errors = check_query_safety_violations(test_dir);

        // Cleanup
        fs::remove_dir_all(test_dir).unwrap();

        // Assert: No errors
        assert!(
            errors.is_empty(),
            "Expected no errors for safe pattern, got: {:?}",
            errors
        );
    }

    /// Test Query Safety: Allows .unwrap() in test code
    #[test]
    fn test_query_safety_allows_test_unwrap() {
        let test_dir = "tests/lints_fixtures/query_test";
        fs::create_dir_all(test_dir).unwrap();

        let test_file = format!("{}/test.rs", test_dir);
        let mut file = fs::File::create(&test_file).unwrap();
        writeln!(
            file,
            r#"
use bevy::prelude::*;

#[test]
fn test_something() {{
    let health = query.get(entity).unwrap();  // ✅ OK in tests
}}
"#
        )
        .unwrap();

        let errors = check_query_safety_violations(test_dir);

        // Cleanup
        fs::remove_dir_all(test_dir).unwrap();

        // Assert: No errors (test code exempted)
        assert!(
            errors.is_empty(),
            "Expected no errors in test code, got: {:?}",
            errors
        );
    }

    /// Test Config Default: Detects missing Default impl
    #[test]
    fn test_config_default_detects_missing() {
        let test_dir = "tests/lints_fixtures/config_no_default";
        fs::create_dir_all(test_dir).unwrap();

        let test_file = format!("{}/test.rs", test_dir);
        let mut file = fs::File::create(&test_file).unwrap();
        writeln!(
            file,
            r#"
use bevy::prelude::*;

#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct MyConfig {{
    pub value: u32,
}}

// Missing: impl Default for MyConfig
"#
        )
        .unwrap();

        let errors = check_config_default_violations(test_dir);

        // Cleanup
        fs::remove_dir_all(test_dir).unwrap();

        // Assert: Should detect 1 error
        assert_eq!(
            errors.len(),
            1,
            "Expected 1 error, got {}: {:?}",
            errors.len(),
            errors
        );
        assert!(
            errors[0].contains("MyConfig") && errors[0].contains("missing 'impl Default'"),
            "Expected error about missing Default, got: {}",
            errors[0]
        );
    }

    /// Test Config Default: Accepts Config with Default
    #[test]
    fn test_config_default_accepts_with_impl() {
        let test_dir = "tests/lints_fixtures/config_with_default";
        fs::create_dir_all(test_dir).unwrap();

        let test_file = format!("{}/test.rs", test_dir);
        let mut file = fs::File::create(&test_file).unwrap();
        writeln!(
            file,
            r#"
use bevy::prelude::*;

#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct MyConfig {{
    pub value: u32,
}}

impl Default for MyConfig {{
    fn default() -> Self {{
        Self {{ value: 10 }}
    }}
}}
"#
        )
        .unwrap();

        let errors = check_config_default_violations(test_dir);

        // Cleanup
        fs::remove_dir_all(test_dir).unwrap();

        // Assert: No errors
        assert!(
            errors.is_empty(),
            "Expected no errors for Config with Default, got: {:?}",
            errors
        );
    }

    /// Test Config Default: Ignores non-Config resources
    #[test]
    fn test_config_default_ignores_non_config() {
        let test_dir = "tests/lints_fixtures/non_config_resource";
        fs::create_dir_all(test_dir).unwrap();

        let test_file = format!("{}/test.rs", test_dir);
        let mut file = fs::File::create(&test_file).unwrap();
        writeln!(
            file,
            r#"
use bevy::prelude::*;

#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct MyResource {{
    pub data: String,
}}

// No Default required (not a Config)
"#
        )
        .unwrap();

        let errors = check_config_default_violations(test_dir);

        // Cleanup
        fs::remove_dir_all(test_dir).unwrap();

        // Assert: No errors (not a Config)
        assert!(
            errors.is_empty(),
            "Expected no errors for non-Config resource, got: {:?}",
            errors
        );
    }

    #[test]
    fn test_system_ordering_detects_missing_in_set() {
        let test_dir = "tests/lints_fixtures/system_ordering_bad";
        fs::create_dir_all(test_dir).unwrap();

        let test_file = format!("{}/plugin.rs", test_dir);
        let mut file = fs::File::create(&test_file).unwrap();
        writeln!(
            file,
            r#"
use bevy::prelude::*;

pub struct TestPlugin;

impl Plugin for TestPlugin {{
    fn build(&self, app: &mut App) {{
        app.add_systems(Update, my_system);  // Missing .in_set()
    }}
}}

fn my_system() {{}}
"#
        )
        .unwrap();

        let errors = check_system_ordering_violations(test_dir);

        fs::remove_dir_all(test_dir).unwrap();

        assert!(
            !errors.is_empty(),
            "Expected error for add_systems without .in_set()"
        );
        assert!(
            errors[0].contains("add_systems without .in_set()"),
            "Expected specific error message, got: {}",
            errors[0]
        );
    }

    #[test]
    fn test_system_ordering_accepts_with_in_set() {
        let test_dir = "tests/lints_fixtures/system_ordering_good";
        fs::create_dir_all(test_dir).unwrap();

        let test_file = format!("{}/plugin.rs", test_dir);
        let mut file = fs::File::create(&test_file).unwrap();
        writeln!(
            file,
            r#"
use bevy::prelude::*;

pub struct TestPlugin;

impl Plugin for TestPlugin {{
    fn build(&self, app: &mut App) {{
        app.add_systems(Update, my_system.in_set(IssunSet::Logic));
    }}
}}

fn my_system() {{}}
"#
        )
        .unwrap();

        let errors = check_system_ordering_violations(test_dir);

        fs::remove_dir_all(test_dir).unwrap();

        assert!(
            errors.is_empty(),
            "Expected no errors for add_systems with .in_set(), got: {:?}",
            errors
        );
    }

    #[test]
    fn test_system_params_detects_world_with_mut_query() {
        let test_dir = "tests/lints_fixtures/system_params_bad";
        fs::create_dir_all(test_dir).unwrap();

        let test_file = format!("{}/systems.rs", test_dir);
        let mut file = fs::File::create(&test_file).unwrap();
        writeln!(
            file,
            r#"
use bevy::prelude::*;

pub fn bad_system(
    mut query: Query<&mut Health>,
    world: &World,
) {{
    // Causes borrowing conflict
}}
"#
        )
        .unwrap();

        let errors = check_system_param_violations(test_dir);

        fs::remove_dir_all(test_dir).unwrap();

        assert!(
            !errors.is_empty(),
            "Expected error for &World with Query<&mut ...>"
        );
        assert!(
            errors[0].contains("borrowing conflicts"),
            "Expected specific error message, got: {}",
            errors[0]
        );
    }

    #[test]
    fn test_system_params_accepts_without_conflicts() {
        let test_dir = "tests/lints_fixtures/system_params_good";
        fs::create_dir_all(test_dir).unwrap();

        let test_file = format!("{}/systems.rs", test_dir);
        let mut file = fs::File::create(&test_file).unwrap();
        writeln!(
            file,
            r#"
use bevy::prelude::*;

pub fn good_system(
    mut query: Query<&mut Health>,
) {{
    // No conflict - uses Query only
}}
"#
        )
        .unwrap();

        let errors = check_system_param_violations(test_dir);

        fs::remove_dir_all(test_dir).unwrap();

        assert!(
            errors.is_empty(),
            "Expected no errors for safe system params, got: {:?}",
            errors
        );
    }

    /// Test Entity::from_bits Safety: Detects unsafe usage
    #[test]
    fn test_entity_from_bits_detects_unsafe_usage() {
        let test_dir = "tests/lints_fixtures/entity_from_bits_unsafe";
        fs::create_dir_all(test_dir).unwrap();

        let test_file = format!("{}/test.rs", test_dir);
        let mut file = fs::File::create(&test_file).unwrap();
        writeln!(
            file,
            r#"
use bevy::prelude::*;

pub fn bad_system(world: &World) {{
    let entity = Entity::from_bits(12345);  // ❌ Unsafe!
    // This entity might be despawned, causing crashes
}}
"#
        )
        .unwrap();

        let errors = check_entity_from_bits_violations(test_dir);

        // Cleanup
        fs::remove_dir_all(test_dir).unwrap();

        // Assert: Should detect 1 error
        assert!(!errors.is_empty(), "Expected errors, got none");
        assert!(
            errors[0].contains("Unsafe Entity::from_bits()"),
            "Expected error about unsafe Entity::from_bits(), got: {}",
            errors[0]
        );
    }

    /// Test Entity::from_bits Safety: Accepts safe wrapper
    #[test]
    fn test_entity_from_bits_accepts_safe_wrapper() {
        let test_dir = "tests/lints_fixtures/entity_from_bits_safe";
        fs::create_dir_all(test_dir).unwrap();

        let test_file = format!("{}/test.rs", test_dir);
        let mut file = fs::File::create(&test_file).unwrap();
        writeln!(
            file,
            r#"
use bevy::prelude::*;

pub fn good_system(world: &World) {{
    // ✅ Safe: uses helper function
    let safe_entity = entity_from_bits_safe(12345, world);
    if safe_entity.exists() {{
        // safe to use
    }}
}}
"#
        )
        .unwrap();

        let errors = check_entity_from_bits_violations(test_dir);

        // Cleanup
        fs::remove_dir_all(test_dir).unwrap();

        // Assert: No errors (using safe wrapper)
        assert!(
            errors.is_empty(),
            "Expected no errors for safe wrapper usage, got: {:?}",
            errors
        );
    }

    /// Test Entity::from_bits Safety: Allows in test code
    #[test]
    fn test_entity_from_bits_allows_in_tests() {
        let test_dir = "tests/lints_fixtures/entity_from_bits_test";
        fs::create_dir_all(test_dir).unwrap();

        let test_file = format!("{}/test.rs", test_dir);
        let mut file = fs::File::create(&test_file).unwrap();
        writeln!(
            file,
            r#"
use bevy::prelude::*;

#[test]
fn test_something() {{
    let entity = Entity::from_bits(12345);  // ✅ OK in tests
}}
"#
        )
        .unwrap();

        let errors = check_entity_from_bits_violations(test_dir);

        // Cleanup
        fs::remove_dir_all(test_dir).unwrap();

        // Assert: No errors (test code exempted)
        assert!(
            errors.is_empty(),
            "Expected no errors in test code, got: {:?}",
            errors
        );
    }
}
