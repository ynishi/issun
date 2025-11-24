//! Event extraction logic for EventReader and EventBus::publish

use crate::types::{EventPublication, EventSubscription};
use syn::{
    visit::Visit, AngleBracketedGenericArguments, Expr, ExprCall, ExprMethodCall, File,
    GenericArgument, Item, PathArguments, Type, TypePath,
};

/// Extract EventReader<E> usage from struct fields
pub fn extract_event_readers(file_path: &str, syntax_tree: &File) -> Vec<EventSubscription> {
    let mut subscriptions = Vec::new();

    for item in &syntax_tree.items {
        if let Item::Struct(item_struct) = item {
            let struct_name = item_struct.ident.to_string();

            // Check each field
            for field in &item_struct.fields {
                if let Some(event_type) = extract_event_reader_type(&field.ty) {
                    // Get approximate line number from span
                    // Note: proc_macro2::Span doesn't expose line numbers in stable Rust
                    // We use 0 as a placeholder
                    let line = 0;

                    subscriptions.push(EventSubscription {
                        subscriber: struct_name.clone(),
                        event_type,
                        file_path: file_path.to_string(),
                        line,
                    });
                }
            }
        }
    }

    subscriptions
}

/// Extract EventBus::publish<E>() calls from function bodies
pub fn extract_event_publications(file_path: &str, syntax_tree: &File) -> Vec<EventPublication> {
    let mut visitor = EventBusVisitor {
        file_path: file_path.to_string(),
        publications: Vec::new(),
        subscriptions: Vec::new(),
        current_function: None,
    };

    visitor.visit_file(syntax_tree);
    visitor.publications
}

/// Extract bus.reader::<E>() calls from function bodies
pub fn extract_reader_calls(file_path: &str, syntax_tree: &File) -> Vec<EventSubscription> {
    let mut visitor = EventBusVisitor {
        file_path: file_path.to_string(),
        publications: Vec::new(),
        subscriptions: Vec::new(),
        current_function: None,
    };

    visitor.visit_file(syntax_tree);
    visitor.subscriptions
}

/// Check if a type is EventReader<E> and extract E
fn extract_event_reader_type(ty: &Type) -> Option<String> {
    if let Type::Path(TypePath { path, .. }) = ty {
        // Check last segment
        if let Some(segment) = path.segments.last() {
            if segment.ident == "EventReader" {
                // Extract generic argument
                if let PathArguments::AngleBracketed(args) = &segment.arguments {
                    if let Some(GenericArgument::Type(Type::Path(type_path))) = args.args.first() {
                        // Get the type name
                        return Some(type_path_to_string(type_path));
                    }
                }
            }
        }
    }
    None
}

/// Convert TypePath to String
fn type_path_to_string(type_path: &TypePath) -> String {
    type_path
        .path
        .segments
        .iter()
        .map(|seg| seg.ident.to_string())
        .collect::<Vec<_>>()
        .join("::")
}

/// Visitor for finding EventBus::publish and reader calls
struct EventBusVisitor {
    file_path: String,
    publications: Vec<EventPublication>,
    subscriptions: Vec<EventSubscription>,
    current_function: Option<String>,
}

impl<'ast> Visit<'ast> for EventBusVisitor {
    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        let old_function = self.current_function.clone();
        self.current_function = Some(node.sig.ident.to_string());

        // Visit function body
        syn::visit::visit_item_fn(self, node);

        self.current_function = old_function;
    }

    fn visit_impl_item_fn(&mut self, node: &'ast syn::ImplItemFn) {
        let old_function = self.current_function.clone();
        self.current_function = Some(node.sig.ident.to_string());

        // Visit method body
        syn::visit::visit_impl_item_fn(self, node);

        self.current_function = old_function;
    }

    fn visit_expr_method_call(&mut self, node: &'ast ExprMethodCall) {
        // Check if receiver looks like EventBus (bus, event_bus, etc.)
        if let Expr::Path(expr_path) = &*node.receiver {
            let receiver_name = expr_path
                .path
                .segments
                .last()
                .map(|s| s.ident.to_string())
                .unwrap_or_default();

            // Common EventBus variable names
            if receiver_name.contains("bus") || receiver_name.contains("events") {
                // Check for publish call
                if node.method == "publish" {
                    // Extract turbofish generic argument: publish::<EventType>()
                    if let Some(event_type) = extract_turbofish_type(&node.turbofish) {
                        // Use placeholder line number
                        let line = 0;

                        self.publications.push(EventPublication {
                            publisher: self
                                .current_function
                                .clone()
                                .unwrap_or_else(|| "unknown".to_string()),
                            event_type,
                            file_path: self.file_path.clone(),
                            line,
                        });
                    }
                }
                // Check for reader call: bus.reader::<EventType>()
                else if node.method == "reader" {
                    if let Some(event_type) = extract_turbofish_type(&node.turbofish) {
                        let line = 0;

                        self.subscriptions.push(EventSubscription {
                            subscriber: self
                                .current_function
                                .clone()
                                .unwrap_or_else(|| "unknown".to_string()),
                            event_type,
                            file_path: self.file_path.clone(),
                            line,
                        });
                    }
                }
            }
        }

        // Continue visiting
        syn::visit::visit_expr_method_call(self, node);
    }

    fn visit_expr_call(&mut self, node: &'ast ExprCall) {
        // Also check for EventBus::publish::<E>() static call pattern
        if let Expr::Path(expr_path) = &*node.func {
            let path_str = expr_path
                .path
                .segments
                .iter()
                .map(|s| s.ident.to_string())
                .collect::<Vec<_>>()
                .join("::");

            if path_str.contains("EventBus") && path_str.ends_with("publish") {
                // Extract generic from last segment
                if let Some(last_seg) = expr_path.path.segments.last() {
                    if let Some(event_type) = extract_segment_turbofish(last_seg) {
                        // Use placeholder line number
                        let line = 0;

                        self.publications.push(EventPublication {
                            publisher: self
                                .current_function
                                .clone()
                                .unwrap_or_else(|| "unknown".to_string()),
                            event_type,
                            file_path: self.file_path.clone(),
                            line,
                        });
                    }
                }
            }
        }

        // Continue visiting
        syn::visit::visit_expr_call(self, node);
    }
}

/// Extract type from turbofish syntax: ::<Type>
fn extract_turbofish_type(turbofish: &Option<AngleBracketedGenericArguments>) -> Option<String> {
    turbofish.as_ref().and_then(|tf| {
        if let Some(GenericArgument::Type(Type::Path(type_path))) = tf.args.first() {
            Some(type_path_to_string(type_path))
        } else {
            None
        }
    })
}

/// Extract type from path segment's generic arguments
fn extract_segment_turbofish(segment: &syn::PathSegment) -> Option<String> {
    if let PathArguments::AngleBracketed(args) = &segment.arguments {
        if let Some(GenericArgument::Type(Type::Path(type_path))) = args.args.first() {
            return Some(type_path_to_string(type_path));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_event_reader_from_simple_struct() {
        let code = r#"
            struct MySystem {
                events: EventReader<MyEvent>,
            }
        "#;

        let syntax_tree = syn::parse_file(code).unwrap();
        let subscriptions = extract_event_readers("test.rs", &syntax_tree);

        assert_eq!(subscriptions.len(), 1);
        assert_eq!(subscriptions[0].subscriber, "MySystem");
        assert_eq!(subscriptions[0].event_type, "MyEvent");
    }

    #[test]
    fn test_extract_publish_from_method() {
        let code = r#"
            impl MySystem {
                fn handle(&self, bus: &mut EventBus) {
                    bus.publish::<MyEvent>(event);
                }
            }
        "#;

        let syntax_tree = syn::parse_file(code).unwrap();
        let publications = extract_event_publications("test.rs", &syntax_tree);

        assert_eq!(publications.len(), 1);
        assert_eq!(publications[0].publisher, "handle");
        assert_eq!(publications[0].event_type, "MyEvent");
    }
}
