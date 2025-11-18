//! Proc macros for ISSUN game engine
//!
//! This crate provides derive macros for:
//! - `#[derive(Scene)]` - Auto-implement Scene trait
//! - `#[derive(Entity)]` - Auto-generate entity methods
//! - `#[derive(Service)]` - Auto-implement Service trait
//! - `#[derive(System)]` - Auto-implement System trait
//! - `#[derive(Asset)]` - Auto-generate asset loading

use proc_macro::TokenStream;
use proc_macro_crate::{crate_name, FoundCrate};
use quote::{format_ident, quote};
use syn::{parse_macro_input, Data, DeriveInput, Fields, Lit};

/// Helper function to get the issun crate identifier
/// Returns `crate` if called from within issun crate, otherwise `::issun`
fn get_crate_name() -> proc_macro2::TokenStream {
    match crate_name("issun") {
        Ok(FoundCrate::Itself) => quote!(crate),
        Ok(FoundCrate::Name(name)) => {
            let ident = format_ident!("{}", name);
            quote!(::#ident)
        }
        Err(_) => quote!(::issun),
    }
}

/// Derive macro for Scene trait
///
/// # Example
/// ```ignore
/// #[derive(Scene)]
/// #[scene(context = "GameContext", initial = "Title(TitleSceneData::new())")]
/// enum MyGameScene {
///     Title(TitleSceneData),
///     Combat(CombatData),
/// }
/// ```
///
/// This will auto-generate:
/// - Scene trait implementation
/// - GameState struct (or custom name via `name` attribute)
/// - GameState::new() with initial scene and context
#[proc_macro_derive(Scene, attributes(scene))]
pub fn derive_scene(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let scene_name = &input.ident;

    // Parse #[scene(...)] attributes
    let scene_attrs = parse_scene_attributes(&input.attrs);

    let crate_name = get_crate_name();

    // Generate Scene trait implementation
    let scene_impl = quote! {
        #[::async_trait::async_trait]
        impl #crate_name::scene::Scene for #scene_name {
            async fn on_enter(
                &mut self,
                _services: &#crate_name::context::ServiceContext,
                _systems: &mut #crate_name::context::SystemContext,
                _resources: &mut #crate_name::context::ResourceContext,
            ) {
                // Default implementation: do nothing
            }

            async fn on_update(
                &mut self,
                _services: &#crate_name::context::ServiceContext,
                _systems: &mut #crate_name::context::SystemContext,
                _resources: &mut #crate_name::context::ResourceContext,
            ) -> #crate_name::scene::SceneTransition<Self> {
                // Default implementation: stay in current scene
                #crate_name::scene::SceneTransition::Stay
            }

            async fn on_exit(
                &mut self,
                _services: &#crate_name::context::ServiceContext,
                _systems: &mut #crate_name::context::SystemContext,
                _resources: &mut #crate_name::context::ResourceContext,
            ) {
                // Default implementation: do nothing
            }

            async fn on_suspend(
                &mut self,
                _services: &#crate_name::context::ServiceContext,
                _systems: &mut #crate_name::context::SystemContext,
                _resources: &mut #crate_name::context::ResourceContext,
            ) {
            }

            async fn on_resume(
                &mut self,
                _services: &#crate_name::context::ServiceContext,
                _systems: &mut #crate_name::context::SystemContext,
                _resources: &mut #crate_name::context::ResourceContext,
            ) {
            }
        }
    };

    // Generate GameState struct if context and initial are specified
    let game_state_gen =
        if let (Some(context), Some(initial)) = (&scene_attrs.context, &scene_attrs.initial) {
            let state_name = scene_attrs
                .name
                .as_ref()
                .map(|n| format_ident!("{}", n))
                .unwrap_or_else(|| format_ident!("GameState"));

            let context_ident = format_ident!("{}", context);
            let initial_expr: proc_macro2::TokenStream = initial.parse().unwrap();

            quote! {
                /// Auto-generated game state combining scene and context
                #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
                pub struct #state_name {
                    pub scene: #scene_name,
                    pub ctx: #context_ident,
                    pub should_quit: bool,
                }

                impl #state_name {
                    pub fn new() -> Self {
                        Self {
                            scene: #scene_name::#initial_expr,
                            ctx: #context_ident::new(),
                            should_quit: false,
                        }
                    }
                }

                impl Default for #state_name {
                    fn default() -> Self {
                        Self::new()
                    }
                }
            }
        } else {
            quote! {}
        };

    // Generate handler dispatcher if handler_params is specified
    let handler_gen = if let Some(params) = &scene_attrs.handler_params {
        let handler_name = scene_attrs
            .handler
            .as_ref()
            .map(|h| format_ident!("{}", h))
            .unwrap_or_else(|| format_ident!("handle_input"));

        let params_tokens: proc_macro2::TokenStream = params.parse().unwrap();

        // Extract parameter names from "ctx: &mut GameContext, input: InputEvent" -> ["ctx", "input"]
        let param_names = extract_param_names(params);

        // Default return type based on scene name (kept for potential future use)
        let _return_type = scene_attrs
            .handler_return
            .as_ref()
            .map(|r| r.parse::<proc_macro2::TokenStream>().unwrap())
            .unwrap_or_else(|| {
                quote! { (#scene_name, ::issun::scene::SceneTransition<#scene_name>) }
            });

        // Extract variants from enum
        let variants = if let Data::Enum(data_enum) = &input.data {
            &data_enum.variants
        } else {
            panic!("Scene derive macro only works on enums");
        };

        // Generate match arms for each variant
        let match_arms = variants.iter().map(|variant| {
            let variant_name = &variant.ident;
            quote! {
                #scene_name::#variant_name(data) => data.#handler_name(
                    services,
                    systems,
                    resources,
                    #(#param_names),*
                ).await
            }
        });

        quote! {
            /// Auto-generated scene input handler dispatcher
            ///
            /// Takes a mutable reference to the scene and returns a transition.
            pub async fn handle_scene_input(
                scene: &mut #scene_name,
                services: &#crate_name::context::ServiceContext,
                systems: &mut #crate_name::context::SystemContext,
                resources: &mut #crate_name::context::ResourceContext,
                #params_tokens,
            ) -> ::issun::scene::SceneTransition<#scene_name> {
                match scene {
                    #(#match_arms),*
                }
            }
        }
    } else {
        quote! {}
    };

    let expanded = quote! {
        #scene_impl
        #game_state_gen
        #handler_gen
    };

    TokenStream::from(expanded)
}

/// Parse #[scene(...)] attributes
struct SceneAttributes {
    context: Option<String>,
    initial: Option<String>,
    name: Option<String>,
    handler: Option<String>,
    handler_params: Option<String>,
    handler_return: Option<String>,
}

fn parse_scene_attributes(attrs: &[syn::Attribute]) -> SceneAttributes {
    let mut context = None;
    let mut initial = None;
    let mut name = None;
    let mut handler = None;
    let mut handler_params = None;
    let mut handler_return = None;

    for attr in attrs {
        if attr.path().is_ident("scene") {
            let _ = attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("context") {
                    if let Ok(value) = meta.value() {
                        if let Ok(Lit::Str(s)) = value.parse::<Lit>() {
                            context = Some(s.value());
                        }
                    }
                } else if meta.path.is_ident("initial") {
                    if let Ok(value) = meta.value() {
                        if let Ok(Lit::Str(s)) = value.parse::<Lit>() {
                            initial = Some(s.value());
                        }
                    }
                } else if meta.path.is_ident("name") {
                    if let Ok(value) = meta.value() {
                        if let Ok(Lit::Str(s)) = value.parse::<Lit>() {
                            name = Some(s.value());
                        }
                    }
                } else if meta.path.is_ident("handler") {
                    if let Ok(value) = meta.value() {
                        if let Ok(Lit::Str(s)) = value.parse::<Lit>() {
                            handler = Some(s.value());
                        }
                    }
                } else if meta.path.is_ident("handler_params") {
                    if let Ok(value) = meta.value() {
                        if let Ok(Lit::Str(s)) = value.parse::<Lit>() {
                            handler_params = Some(s.value());
                        }
                    }
                } else if meta.path.is_ident("handler_return") {
                    if let Ok(value) = meta.value() {
                        if let Ok(Lit::Str(s)) = value.parse::<Lit>() {
                            handler_return = Some(s.value());
                        }
                    }
                }
                Ok(())
            });
        }
    }

    SceneAttributes {
        context,
        initial,
        name,
        handler,
        handler_params,
        handler_return,
    }
}

/// Extract parameter names from function parameters string
/// "ctx: &mut GameContext, input: InputEvent" -> vec![ident("ctx"), ident("input")]
fn extract_param_names(params_str: &str) -> Vec<proc_macro2::Ident> {
    params_str
        .split(',')
        .filter_map(|param| {
            let param = param.trim();
            // Extract the identifier before the colon
            param.split(':').next().map(|name| {
                let name = name.trim();
                format_ident!("{}", name)
            })
        })
        .collect()
}

/// Derive macro for Entity trait
///
/// # Example
/// ```ignore
/// #[derive(Entity)]
/// pub struct Player {
///     #[entity(id)]
///     pub name: String,
///     pub hp: i32,
/// }
/// ```
#[proc_macro_derive(Entity, attributes(entity))]
pub fn derive_entity(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;

    // Extract fields to find #[entity(id)]
    let id_field = match &input.data {
        Data::Struct(data) => {
            match &data.fields {
                Fields::Named(fields) => {
                    // Find field with #[entity(id)] attribute
                    fields
                        .named
                        .iter()
                        .find(|field| {
                            field
                                .attrs
                                .iter()
                                .any(|attr| attr.path().is_ident("entity"))
                        })
                        .map(|field| field.ident.as_ref().unwrap())
                }
                _ => None,
            }
        }
        _ => None,
    };

    let id_impl = if let Some(field_name) = id_field {
        quote! {
            fn id(&self) -> &str {
                &self.#field_name
            }
        }
    } else {
        // No #[entity(id)] found - use default implementation
        quote! {
            fn id(&self) -> &str {
                ""
            }
        }
    };

    let expanded = quote! {
        #[async_trait::async_trait]
        impl Entity for #name {
            #id_impl

            async fn update(&mut self, _ctx: &mut Context) {
                // Default implementation: do nothing
            }
        }
    };

    TokenStream::from(expanded)
}

/// Derive macro for Service trait
///
/// Auto-generates the boilerplate Service trait implementation.
/// Requires #[service(name = "service_name")] attribute.
///
/// # Example
/// ```ignore
/// use crate::service::Service; // or use issun::service::Service;
///
/// #[derive(Service)]
/// #[service(name = "combat_service")]
/// pub struct CombatService {
///     min_damage: i32,
/// }
/// ```
///
/// This generates:
/// - name() method returning the specified service name
/// - as_any() and as_any_mut() for downcasting
/// - async_trait wrapper
///
/// Note: You must have `Service` trait in scope via `use` statement.
#[proc_macro_derive(Service, attributes(service))]
pub fn derive_service(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let struct_name = &input.ident;

    // Parse #[service(name = "...")] attribute
    let service_name = parse_service_name(&input.attrs);
    let service_name_lit = syn::LitStr::new(&service_name, proc_macro2::Span::call_site());

    let crate_name = get_crate_name();

    let expanded = quote! {
        impl #struct_name {
            pub const NAME: &'static str = #service_name_lit;
        }

        #[::async_trait::async_trait]
        impl #crate_name::service::Service for #struct_name {
            fn name(&self) -> &'static str {
                #service_name
            }

            fn clone_box(&self) -> Box<dyn #crate_name::service::Service> {
                Box::new(self.clone())
            }

            fn as_any(&self) -> &dyn ::std::any::Any {
                self
            }

            fn as_any_mut(&mut self) -> &mut dyn ::std::any::Any {
                self
            }
        }
    };

    TokenStream::from(expanded)
}

/// Parse #[service(name = "service_name")] attribute
fn parse_service_name(attrs: &[syn::Attribute]) -> String {
    for attr in attrs {
        if attr.path().is_ident("service") {
            let mut name = None;
            let _ = attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("name") {
                    if let Ok(value) = meta.value() {
                        if let Ok(Lit::Str(s)) = value.parse::<Lit>() {
                            name = Some(s.value());
                        }
                    }
                }
                Ok(())
            });
            if let Some(n) = name {
                return n;
            }
        }
    }

    // Default: use lowercase struct name
    "unknown_service".to_string()
}

/// Derive macro for System trait
///
/// Auto-generates the boilerplate System trait implementation.
/// Requires #[system(name = "system_name")] attribute.
///
/// # Example
/// ```ignore
/// use crate::system::System; // or use issun::system::System;
///
/// #[derive(System)]
/// #[system(name = "combat_engine")]
/// pub struct CombatSystem {
///     turn_count: u32,
///     log: Vec<String>,
/// }
/// ```
///
/// This generates:
/// - name() method returning the specified system name
/// - as_any() and as_any_mut() for downcasting
/// - async_trait wrapper
///
/// Note: You must have `System` trait in scope via `use` statement.
#[proc_macro_derive(System, attributes(system))]
pub fn derive_system(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let struct_name = &input.ident;

    // Parse #[system(name = "...")] attribute
    let system_name = parse_system_name(&input.attrs);
    let system_name_lit = syn::LitStr::new(&system_name, proc_macro2::Span::call_site());

    let crate_name = get_crate_name();

    let expanded = quote! {
        impl #struct_name {
            pub const NAME: &'static str = #system_name_lit;
        }

        #[::async_trait::async_trait]
        impl #crate_name::system::System for #struct_name {
            fn name(&self) -> &'static str {
                #system_name
            }

            fn as_any(&self) -> &dyn ::std::any::Any {
                self
            }

            fn as_any_mut(&mut self) -> &mut dyn ::std::any::Any {
                self
            }
        }
    };

    TokenStream::from(expanded)
}

/// Parse #[system(name = "system_name")] attribute
fn parse_system_name(attrs: &[syn::Attribute]) -> String {
    for attr in attrs {
        if attr.path().is_ident("system") {
            let mut name = None;
            let _ = attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("name") {
                    if let Ok(value) = meta.value() {
                        if let Ok(Lit::Str(s)) = value.parse::<Lit>() {
                            name = Some(s.value());
                        }
                    }
                }
                Ok(())
            });
            if let Some(n) = name {
                return n;
            }
        }
    }

    // Default: use lowercase struct name
    "unknown_system".to_string()
}

/// Derive macro for Asset trait
///
/// # Example
/// ```ignore
/// #[derive(Asset)]
/// pub struct EnemyAsset {
///     pub name: &'static str,
///     pub hp: i32,
/// }
/// ```
#[proc_macro_derive(Asset)]
pub fn derive_asset(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;

    let expanded = quote! {
        impl Asset for #name {
            // Uses default implementation from trait
        }
    };

    TokenStream::from(expanded)
}

/// Derive macro for Resource trait
///
/// Automatically implements the `Resource` marker trait for types that
/// can be stored in the global resource registry.
///
/// # Example
/// ```ignore
/// use issun::prelude::*;
///
/// #[derive(Resource)]
/// pub struct GameConfig {
///     pub fps: u32,
///     pub difficulty: f32,
/// }
///
/// #[derive(Resource)]
/// pub struct EnemyDatabase {
///     pub enemies: Vec<EnemyAsset>,
/// }
/// ```
#[proc_macro_derive(Resource)]
pub fn derive_resource(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;
    let crate_name = get_crate_name();

    let expanded = quote! {
        impl #crate_name::resources::Resource for #name {
            // Uses default implementation from trait
        }
    };

    TokenStream::from(expanded)
}
