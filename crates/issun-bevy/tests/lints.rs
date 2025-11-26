//! Reflect Linting Test
//!
//! Ensures all Component/Resource/Message/Event types have #[derive(Reflect)]
//! and #[reflect(...)] attributes for modding and save/load support.
//!
//! ⚠️ IMPORTANT: Bevy 0.17 Message types only require #[derive(Reflect)]
//! - ReflectMessage helper does NOT exist in Bevy 0.17
//! - Message trait is just: `Send + Sync + 'static`
//! - #[reflect(Message)] is not needed (and will cause compile errors)

use std::fs;
use std::path::Path;
use syn::{visit::Visit, ItemStruct};
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

                // ⚠️ CRITICAL: Bevy 0.17 doesn't have ReflectMessage
                // Message types only need #[derive(Reflect)], not #[reflect(Message)]
                // See: https://github.com/bevyengine/bevy/discussions/11587
                if *ty != "Message" && !self.has_reflect_attr(node, ty) {
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

        // Assert: 7 errors total
        // - Component: 2 errors (#[derive(Reflect)] + #[reflect(Component)])
        // - Resource: 2 errors (#[derive(Reflect)] + #[reflect(Resource)])
        // - Message: 1 error (#[derive(Reflect)] only, no #[reflect(Message)])
        // - Event: 2 errors (#[derive(Reflect)] + #[reflect(Event)])
        assert_eq!(
            errors.len(),
            7,
            "Expected 7 errors, got {}: {:?}",
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

        // Event
        assert!(errors
            .iter()
            .any(|e| e.contains("BadEvent") && e.contains("missing #[derive(Reflect)]")));
        assert!(errors
            .iter()
            .any(|e| e.contains("BadEvent") && e.contains("missing #[reflect(Event)]")));
    }
}
