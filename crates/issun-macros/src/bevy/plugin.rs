//! IssunBevyPlugin derive macro
//!
//! Auto-generates Plugin implementation with resource registration and builder methods.
//!
//! # Requirements
//!
//! **All fields marked with `#[resource]` or `#[config]` must implement `Clone`:**
//!
//! ```ignore
//! #[derive(Resource, Default, Clone)]  // ← Clone required!
//! pub struct GameStats {
//!     pub score: u32,
//! }
//! ```
//!
//! This is because `Plugin::build(&self, app)` takes `&self`, so resources
//! must be cloned when calling `app.insert_resource(self.field.clone())`.
//!
//! # Example
//! ```ignore
//! use bevy::prelude::*;
//! use issun_macros::IssunBevyPlugin;
//!
//! #[derive(Resource, Clone, Default)]
//! pub struct MyConfig {
//!     pub difficulty: f32,
//! }
//!
//! #[derive(Resource, Clone, Default)]
//! pub struct GameStats {
//!     pub score: u32,
//! }
//!
//! #[derive(Default, IssunBevyPlugin)]
//! #[plugin(name = "my_plugin")]
//! pub struct MyPlugin {
//!     #[config]
//!     pub config: MyConfig,
//!
//!     #[resource]
//!     pub stats: GameStats,
//! }
//!
//! // Usage
//! App::new()
//!     .add_plugins(MyPlugin::default()
//!         .with_config(MyConfig { difficulty: 2.0 })
//!     );
//! ```
//!
//! # Generated Code
//!
//! This generates:
//! - `Plugin::build()` implementation with resource registration
//! - Builder methods: `with_config()`, `with_stats()`
//!
//! # Attributes
//!
//! ## Struct-level attributes
//!
//! - `#[plugin(name = "...")]` - Custom plugin name (optional)
//! - `#[plugin(auto_register_types = true)]` - Auto-register all resource types for Reflection (optional)
//! - `#[plugin(messages = [Type1, Type2, ...])]` - Auto-register messages (optional)
//! - `#[plugin(components = [Type1, Type2, ...])]` - Auto-register component types for Reflection (optional, Phase 2.2)
//! - `#[plugin(startup_systems = [fn1, fn2, ...])]` - Auto-register systems to Startup schedule (optional, Phase 2.2)
//! - `#[plugin(update_systems = [fn1, fn2, ...])]` - Auto-register systems to Update schedule (optional, Phase 2.2)
//! - `#[plugin(requires = [Plugin1, Plugin2, ...])]` - Declare issun-bevy plugin dependencies (optional, Phase 2.3)
//! - `#[plugin(requires_bevy = [BevyPlugin1, ...])]` - Declare Bevy standard plugin dependencies (optional, Phase 2.3)
//! - `#[plugin(auto_require_core = true)]` - Auto-require IssunCorePlugin (default: true, Phase 2.3)
//!
//! ## Field-level attributes
//!
//! - `#[config]` - Mark as config resource (insert_resource + builder method)
//! - `#[resource]` - Mark as resource (insert_resource + builder method)
//! - `#[skip]` - Skip this field (no auto-generation)

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, DeriveInput, Data, Fields, Field, Attribute, Lit, Type, Token, Path};
use syn::parse::Parse;

/// Plugin configuration options
#[derive(Default)]
struct PluginConfig {
    name: String,
    auto_register_types: bool,
    messages: Vec<Type>,
    components: Vec<Type>,         // Phase 2.2
    startup_systems: Vec<Path>,    // Phase 2.2
    update_systems: Vec<Path>,     // Phase 2.2
    requires: Vec<Type>,            // Phase 2.3 - issun-bevy plugin dependencies
    requires_bevy: Vec<Type>,       // Phase 2.3 - Bevy standard plugin dependencies
    auto_require_core: bool,        // Phase 2.3 - Auto-require IssunCorePlugin (default: true)
}

/// Helper struct for parsing messages = [Type1, Type2, ...]
struct MessageList {
    types: Vec<Type>,
}

impl Parse for MessageList {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let content;
        syn::bracketed!(content in input);
        let types = content.parse_terminated(Type::parse, Token![,])?;
        Ok(MessageList {
            types: types.into_iter().collect(),
        })
    }
}

/// Helper struct for parsing systems = [fn1, fn2, ...]
struct PathList {
    paths: Vec<Path>,
}

impl Parse for PathList {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let content;
        syn::bracketed!(content in input);
        let paths = content.parse_terminated(Path::parse, Token![,])?;
        Ok(PathList {
            paths: paths.into_iter().collect(),
        })
    }
}

/// Derive macro for auto-generating Bevy Plugin boilerplate
pub fn derive_issun_bevy_plugin_impl(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let struct_name = &input.ident;

    // Parse #[plugin(...)] attributes
    let plugin_config = parse_plugin_attrs(&input.attrs, struct_name);

    // Parse fields
    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => {
                return syn::Error::new_spanned(
                    &input,
                    "IssunBevyPlugin only supports structs with named fields",
                )
                .to_compile_error()
                .into();
            }
        },
        _ => {
            return syn::Error::new_spanned(
                &input,
                "IssunBevyPlugin only supports structs",
            )
            .to_compile_error()
            .into();
        }
    };

    // Collect fields marked with #[config] or #[resource]
    let mut config_fields = Vec::new();
    let mut resource_fields = Vec::new();

    for field in fields.iter() {
        if has_skip_attr(field) {
            continue;
        }

        if has_config_attr(field) {
            config_fields.push(field);
        } else if has_resource_attr(field) {
            resource_fields.push(field);
        }
    }

    // Generate resource registration code
    let resource_registrations: Vec<_> = config_fields
        .iter()
        .chain(resource_fields.iter())
        .map(|field| {
            let field_name = field.ident.as_ref().unwrap();
            quote! {
                app.insert_resource(self.#field_name.clone());
            }
        })
        .collect();

    // Generate type registration code (if auto_register_types = true)
    let type_registrations = if plugin_config.auto_register_types {
        let types: Vec<_> = config_fields
            .iter()
            .chain(resource_fields.iter())
            .map(|field| {
                let field_type = &field.ty;
                quote! {
                    app.register_type::<#field_type>();
                }
            })
            .collect();

        quote! {
            // Type registration for Reflection (auto_register_types = true)
            #(#types)*
        }
    } else {
        quote! {}
    };

    // Generate message registration code
    let message_registrations = if !plugin_config.messages.is_empty() {
        let messages = &plugin_config.messages;
        quote! {
            // Message/Event registration
            #(app.add_event::<#messages>();)*
        }
    } else {
        quote! {}
    };

    // Generate component registration code (Phase 2.2)
    let component_registrations = if !plugin_config.components.is_empty() {
        let components = &plugin_config.components;
        quote! {
            // Component registration (Phase 2.2)
            #(app.register_type::<#components>();)*
        }
    } else {
        quote! {}
    };

    // Generate startup systems code (Phase 2.2)
    let startup_systems = if !plugin_config.startup_systems.is_empty() {
        let systems = &plugin_config.startup_systems;
        quote! {
            // Startup systems (Phase 2.2)
            app.add_systems(::bevy::prelude::Startup, (#(#systems),*));
        }
    } else {
        quote! {}
    };

    // Generate update systems code (Phase 2.2)
    let update_systems = if !plugin_config.update_systems.is_empty() {
        let systems = &plugin_config.update_systems;
        quote! {
            // Update systems (Phase 2.2)
            app.add_systems(::bevy::prelude::Update, (#(#systems),*));
        }
    } else {
        quote! {}
    };

    // Generate dependency checks (Phase 2.3)
    let dependency_checks = generate_dependency_checks(&plugin_config, struct_name);

    // Generate marker resource (Phase 2.3)
    let marker_name = format_ident!("{}Marker", struct_name);
    let marker_registration = quote! {
        // Marker resource registration (Phase 2.3)
        #[derive(::bevy::prelude::Resource)]
        pub struct #marker_name;
    };
    let marker_insert = quote! {
        app.insert_resource(#marker_name);
    };

    // Generate builder methods
    let builder_methods: Vec<_> = config_fields
        .iter()
        .chain(resource_fields.iter())
        .map(|field| {
            let field_name = field.ident.as_ref().unwrap();
            let field_type = &field.ty;
            let method_name = format_ident!("with_{}", field_name);

            // Extract doc comment if exists
            let doc = extract_field_doc(field);
            let doc_attr = if let Some(doc_text) = doc {
                quote! { #[doc = #doc_text] }
            } else {
                let default_doc = format!("Set {}", field_name);
                quote! { #[doc = #default_doc] }
            };

            quote! {
                #doc_attr
                pub fn #method_name(mut self, #field_name: #field_type) -> Self {
                    self.#field_name = #field_name;
                    self
                }
            }
        })
        .collect();

    let expanded = quote! {
        // Marker resource definition (Phase 2.3)
        #marker_registration

        impl ::bevy::prelude::Plugin for #struct_name {
            fn build(&self, app: &mut ::bevy::prelude::App) {
                // Dependency checks (Phase 2.3)
                #dependency_checks

                // Marker registration (Phase 2.3)
                #marker_insert

                // Auto-generated resource registration
                // Note: All resources must implement Clone
                #(#resource_registrations)*

                #type_registrations

                #message_registrations

                #component_registrations

                #startup_systems

                #update_systems

                // Extension point for user customization
                // Add your systems and additional setup below:
                // app.add_systems(Update, your_system);
            }
        }

        impl #struct_name {
            #(#builder_methods)*
        }
    };

    TokenStream::from(expanded)
}

/// Parse #[plugin(...)] attributes
fn parse_plugin_attrs(attrs: &[Attribute], struct_name: &syn::Ident) -> PluginConfig {
    let mut config = PluginConfig {
        name: to_snake_case(&struct_name.to_string()),
        auto_register_types: false,
        messages: Vec::new(),
        components: Vec::new(),
        startup_systems: Vec::new(),
        update_systems: Vec::new(),
        requires: Vec::new(),
        requires_bevy: Vec::new(),
        auto_require_core: true,  // Default: true
    };

    for attr in attrs {
        if !attr.path().is_ident("plugin") {
            continue;
        }

        let _ = attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("name") {
                if let Ok(value) = meta.value() {
                    if let Ok(Lit::Str(s)) = value.parse::<Lit>() {
                        config.name = s.value();
                    }
                }
            } else if meta.path.is_ident("auto_register_types") {
                if let Ok(value) = meta.value() {
                    if let Ok(Lit::Bool(b)) = value.parse::<Lit>() {
                        config.auto_register_types = b.value();
                    }
                }
            } else if meta.path.is_ident("messages") {
                // Parse messages = [Type1, Type2, ...]
                if let Ok(value) = meta.value() {
                    if let Ok(messages) = value.parse::<MessageList>() {
                        config.messages = messages.types;
                    }
                }
            } else if meta.path.is_ident("components") {
                // Parse components = [Type1, Type2, ...]
                if let Ok(value) = meta.value() {
                    if let Ok(components) = value.parse::<MessageList>() {
                        config.components = components.types;
                    }
                }
            } else if meta.path.is_ident("startup_systems") {
                // Parse startup_systems = [fn1, fn2, ...]
                if let Ok(value) = meta.value() {
                    if let Ok(systems) = value.parse::<PathList>() {
                        config.startup_systems = systems.paths;
                    }
                }
            } else if meta.path.is_ident("update_systems") {
                // Parse update_systems = [fn1, fn2, ...]
                if let Ok(value) = meta.value() {
                    if let Ok(systems) = value.parse::<PathList>() {
                        config.update_systems = systems.paths;
                    }
                }
            } else if meta.path.is_ident("requires") {
                // Parse requires = [Plugin1, Plugin2, ...]
                if let Ok(value) = meta.value() {
                    if let Ok(plugins) = value.parse::<MessageList>() {
                        config.requires = plugins.types;
                    }
                }
            } else if meta.path.is_ident("requires_bevy") {
                // Parse requires_bevy = [BevyPlugin1, ...]
                if let Ok(value) = meta.value() {
                    if let Ok(plugins) = value.parse::<MessageList>() {
                        config.requires_bevy = plugins.types;
                    }
                }
            } else if meta.path.is_ident("auto_require_core") {
                // Parse auto_require_core = true/false
                if let Ok(value) = meta.value() {
                    if let Ok(Lit::Bool(b)) = value.parse::<Lit>() {
                        config.auto_require_core = b.value();
                    }
                }
            }
            Ok(())
        });
    }

    config
}

/// Check if field has #[config] attribute
fn has_config_attr(field: &Field) -> bool {
    field.attrs.iter().any(|attr| attr.path().is_ident("config"))
}

/// Check if field has #[resource] attribute
fn has_resource_attr(field: &Field) -> bool {
    field.attrs.iter().any(|attr| attr.path().is_ident("resource"))
}

/// Check if field has #[skip] attribute
fn has_skip_attr(field: &Field) -> bool {
    field.attrs.iter().any(|attr| attr.path().is_ident("skip"))
}

/// Extract doc comment from field
fn extract_field_doc(field: &Field) -> Option<String> {
    for attr in &field.attrs {
        if attr.path().is_ident("doc") {
            if let Ok(Lit::Str(s)) = attr.parse_args::<Lit>() {
                return Some(s.value().trim().to_string());
            }
        }
    }
    None
}

/// Generate dependency check code (Phase 2.3)
fn generate_dependency_checks(config: &PluginConfig, struct_name: &syn::Ident) -> proc_macro2::TokenStream {
    let mut checks = Vec::new();
    let struct_name_str = struct_name.to_string();

    // IssunCorePlugin check (if auto_require_core)
    if config.auto_require_core {
        checks.push(quote! {
            assert!(
                app.world().contains_resource::<::issun_bevy::IssunCorePluginMarker>(),
                "{} requires IssunCorePlugin. Add it before this plugin:\n\
                 app.add_plugins(::issun_bevy::IssunCorePlugin);\n\
                 app.add_plugins({}::default());",
                #struct_name_str,
                #struct_name_str
            );
        });
    }

    // issun-bevy plugin checks
    for plugin_type in &config.requires {
        // Extract type name for marker (e.g., TimePlugin -> TimePluginMarker)
        // Use quote to convert Type to string, then extract last segment
        let type_str = quote!(#plugin_type).to_string();
        let type_name = type_str.split("::").last().unwrap_or(&type_str);
        let type_name = type_name.trim();
        let marker_name = format_ident!("{}Marker", type_name);
        let plugin_str = quote!(#plugin_type).to_string();

        checks.push(quote! {
            assert!(
                app.world().contains_resource::<#marker_name>(),
                "{} requires {}. Add it before this plugin:\n\
                 app.add_plugins({}::default());\n\
                 app.add_plugins({}::default());",
                #struct_name_str,
                #plugin_str,
                #plugin_str,
                #struct_name_str
            );
        });
    }

    // Bevy plugin checks (currently just warn, as detection is complex)
    if !config.requires_bevy.is_empty() {
        let bevy_plugins: Vec<String> = config.requires_bevy.iter()
            .map(|t| quote!(#t).to_string())
            .collect();
        let _bevy_list = bevy_plugins.join(", ");

        checks.push(quote! {
            // Note: Bevy plugin detection is not implemented yet
            // Please ensure the following Bevy plugins are added: #bevy_list
        });
    }

    quote! {
        #(#checks)*
    }
}

/// Convert PascalCase to snake_case
fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    for (i, ch) in s.chars().enumerate() {
        if i > 0 && ch.is_uppercase() {
            result.push('_');
        }
        result.push(ch.to_ascii_lowercase());
    }
    result
}
