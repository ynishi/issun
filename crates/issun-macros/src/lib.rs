//! Proc macros for ISSUN game engine
//!
//! This crate provides derive macros for:
//! - `#[derive(Scene)]` - Auto-implement Scene trait
//! - `#[derive(Entity)]` - Auto-generate entity methods
//! - `#[derive(Service)]` - Auto-implement Service trait
//! - `#[derive(System)]` - Auto-implement System trait
//! - `#[derive(Asset)]` - Auto-generate asset loading

use proc_macro::TokenStream;
use proc_macro2::Span;
use proc_macro_crate::{crate_name, FoundCrate};
use quote::{format_ident, quote, ToTokens};
use std::collections::HashMap;
use std::mem;
use syn::{
    braced,
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    spanned::Spanned,
    Attribute, Block, Data, DeriveInput, Fields, FnArg, Ident, ImplItem, ImplItemFn, ItemFn,
    ItemImpl, Lit, LitStr, Meta, Pat, PatIdent, PatType, Path, Result, Signature, Stmt, Token,
    Type, Visibility,
};

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

/// Derive macro for Plugin trait
///
/// # Example
/// ```ignore
/// #[derive(Default, Plugin)]
/// #[plugin(name = "my_plugin")]
/// #[plugin(service = MyService)]
/// #[plugin(system = MySystem)]
/// #[plugin(state = MyState)]
/// pub struct MyPlugin;
/// ```
#[proc_macro_derive(Plugin, attributes(plugin))]
pub fn derive_plugin(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let crate_name = get_crate_name();

    // Parse plugin attributes
    let mut plugin_name = None;
    let mut services = Vec::new();
    let mut systems = Vec::new();
    let mut states = Vec::new();
    let mut resources = Vec::new();

    for attr in &input.attrs {
        if !attr.path().is_ident("plugin") {
            continue;
        }

        let result: Result<()> = attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("name") {
                let value = meta.value()?;
                let lit: LitStr = value.parse()?;
                plugin_name = Some(lit.value());
                Ok(())
            } else if meta.path.is_ident("service") {
                let value = meta.value()?;
                let ty: Type = value.parse()?;
                services.push(ty);
                Ok(())
            } else if meta.path.is_ident("system") {
                let value = meta.value()?;
                let ty: Type = value.parse()?;
                systems.push(ty);
                Ok(())
            } else if meta.path.is_ident("state") {
                let value = meta.value()?;
                let ty: Type = value.parse()?;
                states.push(ty);
                Ok(())
            } else if meta.path.is_ident("resource") {
                let value = meta.value()?;
                let ty: Type = value.parse()?;
                resources.push(ty);
                Ok(())
            } else {
                Err(meta.error("expected `name`, `service`, `system`, `state`, or `resource`"))
            }
        });

        if let Err(err) = result {
            return err.to_compile_error().into();
        }
    }

    let plugin_name = plugin_name.unwrap_or_else(|| {
        // Default: convert MyPlugin -> "my_plugin"
        let name_str = name.to_string();
        name_str
            .trim_end_matches("Plugin")
            .chars()
            .enumerate()
            .flat_map(|(i, c)| {
                if i > 0 && c.is_uppercase() {
                    vec!['_', c.to_ascii_lowercase()]
                } else {
                    vec![c.to_ascii_lowercase()]
                }
            })
            .collect::<String>()
    });

    // Generate service registrations
    let service_registrations = services.iter().map(|ty| {
        quote! {
            builder.register_service(Box::new(#ty::default()));
        }
    });

    // Generate system registrations
    let system_registrations = systems.iter().map(|ty| {
        quote! {
            builder.register_system(Box::new(#ty::default()));
        }
    });

    // Generate state registrations
    let state_registrations = states.iter().map(|ty| {
        quote! {
            builder.register_runtime_state(#ty::default());
        }
    });

    // Generate resource registrations
    let resource_registrations = resources.iter().map(|ty| {
        quote! {
            builder.register_resource(#ty::default());
        }
    });

    let expanded = quote! {
        #[::async_trait::async_trait]
        impl #crate_name::plugin::Plugin for #name {
            fn name(&self) -> &'static str {
                #plugin_name
            }

            fn build(&self, builder: &mut dyn #crate_name::plugin::PluginBuilder) {
                use #crate_name::plugin::PluginBuilderExt;
                #(#service_registrations)*
                #(#system_registrations)*
                #(#state_registrations)*
                #(#resource_registrations)*
            }
        }
    };

    TokenStream::from(expanded)
}

/// Attribute macro that generates `process_events` for systems reacting to events.
#[proc_macro_attribute]
pub fn event_handler(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as EventHandlerArgs);
    let mut item_impl = parse_macro_input!(item as ItemImpl);

    if item_impl.trait_.is_some() {
        return syn::Error::new(
            item_impl.impl_token.span(),
            "#[event_handler] can only be used on inherent impl blocks",
        )
        .to_compile_error()
        .into();
    }

    let crate_name = get_crate_name();
    let mut context = match EventHandlerContext::new(args) {
        Ok(ctx) => ctx,
        Err(err) => return err.to_compile_error().into(),
    };

    for impl_item in &mut item_impl.items {
        if let ImplItem::Fn(method) = impl_item {
            let mut subscribe_attr = None;
            method.attrs.retain(|attr| {
                if attr.path().is_ident("subscribe") {
                    subscribe_attr = Some(attr.clone());
                    false
                } else {
                    true
                }
            });

            if let Some(attr) = subscribe_attr {
                if let Err(err) = context.register_handler(method, attr) {
                    return err.to_compile_error().into();
                }
            }
        }
    }

    if context.handlers.is_empty() {
        return syn::Error::new(
            item_impl.impl_token.span(),
            "#[event_handler] requires at least one #[subscribe] method",
        )
        .to_compile_error()
        .into();
    }

    match context.generate_process_fn(&crate_name) {
        Ok(process_fn) => {
            item_impl.items.push(ImplItem::Fn(process_fn));
            TokenStream::from(quote! { #item_impl })
        }
        Err(err) => err.to_compile_error().into(),
    }
}

#[derive(Default)]
struct EventHandlerArgs {
    default_state: Option<DefaultState>,
}

struct DefaultState {
    ty: Type,
    repr: String,
}

impl Parse for EventHandlerArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut args = EventHandlerArgs::default();

        while !input.is_empty() {
            let ident: Ident = input.parse()?;
            input.parse::<Token![=]>()?;

            match ident.to_string().as_str() {
                "state" | "default_state" => {
                    let ty = parse_type_value(input)?;
                    let repr = type_to_key(&ty);
                    args.default_state = Some(DefaultState { ty, repr });
                }
                "system" => {
                    // Consume the value for validation but ignore it for now.
                    let _ = parse_type_value(input)?;
                }
                other => {
                    return Err(syn::Error::new(
                        ident.span(),
                        format!("Unknown event_handler attribute key `{}`", other),
                    ));
                }
            }

            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(args)
    }
}

struct EventHandlerContext {
    default_state: Option<DefaultState>,
    events: Vec<EventCollection>,
    event_lookup: HashMap<String, usize>,
    handlers: Vec<Handler>,
    uses_services: bool,
}

impl EventHandlerContext {
    fn new(args: EventHandlerArgs) -> Result<Self> {
        Ok(Self {
            default_state: args.default_state,
            events: Vec::new(),
            event_lookup: HashMap::new(),
            handlers: Vec::new(),
            uses_services: false,
        })
    }

    fn register_handler(
        &mut self,
        method: &mut ImplItemFn,
        subscribe_attr: Attribute,
    ) -> Result<()> {
        if method.sig.asyncness.is_none() {
            return Err(syn::Error::new(
                method.sig.fn_token.span(),
                "#[subscribe] handlers must be async",
            ));
        }

        if method.sig.inputs.is_empty() {
            return Err(syn::Error::new(
                method.sig.fn_token.span(),
                "Event handler must accept &mut self",
            ));
        }

        let mut inputs_iter = method.sig.inputs.iter_mut();

        match inputs_iter.next() {
            Some(FnArg::Receiver(receiver)) => {
                if receiver.mutability.is_none() {
                    return Err(syn::Error::new(
                        receiver.self_token.span(),
                        "event handlers must take `&mut self`",
                    ));
                }
            }
            _ => {
                return Err(syn::Error::new(
                    method.sig.fn_token.span(),
                    "event handlers must start with `&mut self`",
                ));
            }
        }

        let subscribe = parse_subscribe_attr(subscribe_attr)?;
        let event_index = self.register_event(&subscribe.event_type);

        let event_input = inputs_iter.next().ok_or_else(|| {
            syn::Error::new(
                method.sig.fn_token.span(),
                "event handler must accept event parameter",
            )
        })?;

        let event_ty = match event_input {
            FnArg::Typed(pat_type) => match pat_type.ty.as_ref() {
                Type::Reference(reference) => {
                    if reference.mutability.is_some() {
                        return Err(syn::Error::new(
                            reference.and_token.span(),
                            "event parameter must be `&EventType`",
                        ));
                    }
                    reference.elem.as_ref().clone()
                }
                _ => {
                    return Err(syn::Error::new(
                        pat_type.ty.span(),
                        "event parameter must be a reference",
                    ))
                }
            },
            _ => {
                return Err(syn::Error::new(
                    method.sig.fn_token.span(),
                    "event parameter must be named",
                ))
            }
        };

        let requested_event = subscribe.event_type.to_token_stream().to_string();
        let actual_event = event_ty.to_token_stream().to_string();
        if requested_event != actual_event {
            return Err(syn::Error::new(
                method.sig.ident.span(),
                "event parameter type must match #[subscribe(...)]",
            ));
        }

        let mut args = Vec::new();
        for input in inputs_iter {
            let pat_type = match input {
                FnArg::Typed(pat_type) => pat_type,
                FnArg::Receiver(_) => {
                    return Err(syn::Error::new(
                        input.span(),
                        "unexpected self parameter in handler",
                    ))
                }
            };

            let arg = parse_handler_arg(pat_type, self.default_state.as_ref())?;
            if matches!(arg.kind, HandlerArgKind::Service { .. }) {
                self.uses_services = true;
            }
            args.push(arg);
        }

        self.handlers.push(Handler {
            method_ident: method.sig.ident.clone(),
            event_index,
            filter: subscribe.filter,
            args,
        });

        Ok(())
    }

    fn register_event(&mut self, ty: &Type) -> usize {
        let key = type_to_key(ty);
        if let Some(index) = self.event_lookup.get(&key) {
            *index
        } else {
            let ident = format_ident!("__events_{}", sanitize_ident(&key));
            let index = self.events.len();
            self.events.push(EventCollection {
                ty: ty.clone(),
                ident,
            });
            self.event_lookup.insert(key, index);
            index
        }
    }

    fn generate_process_fn(&self, crate_name: &proc_macro2::TokenStream) -> Result<ImplItemFn> {
        let event_bus_ty = quote! { #crate_name::event::EventBus };
        let resource_ctx_ty = quote! { #crate_name::context::ResourceContext };
        let service_ctx_ty = quote! { #crate_name::context::ServiceContext };

        let collects = self.events.iter().map(|event| {
            let ident = &event.ident;
            let ty = &event.ty;
            quote! {
                let #ident: ::std::vec::Vec<#ty> =
                    event_bus.reader::<#ty>().iter().cloned().collect();
            }
        });

        let empty_check = if self.events.is_empty() {
            quote! {}
        } else {
            let empties = self.events.iter().map(|event| {
                let ident = &event.ident;
                quote! { #ident.is_empty() }
            });
            quote! {
                if true #(&& #empties)* {
                    return;
                }
            }
        };

        let handler_blocks = self
            .handlers
            .iter()
            .map(|handler| handler.expand(self.events.as_slice()));

        let service_usage = if self.uses_services {
            quote! {}
        } else {
            quote! { let _ = services; }
        };

        let body = quote! {
            let mut event_bus = match resources.get_mut::<#event_bus_ty>().await {
                Some(bus) => bus,
                None => return,
            };

            #(#collects)*

            #empty_check

            drop(event_bus);

            #service_usage
            #(#handler_blocks)*
        };

        let process_fn: ImplItemFn = syn::parse_quote! {
            pub async fn process_events(
                &mut self,
                services: &#service_ctx_ty,
                resources: &mut #resource_ctx_ty,
            ) {
                #body
            }
        };

        Ok(process_fn)
    }
}

struct EventCollection {
    ty: Type,
    ident: Ident,
}

struct Handler {
    method_ident: Ident,
    event_index: usize,
    filter: Option<Ident>,
    args: Vec<HandlerArg>,
}

impl Handler {
    fn expand(&self, events: &[EventCollection]) -> proc_macro2::TokenStream {
        let event_ident = &events[self.event_index].ident;
        let method_ident = &self.method_ident;
        let filter_check = if let Some(filter) = &self.filter {
            quote! {
                if !self.#filter(event) {
                    continue;
                }
            }
        } else {
            quote! {}
        };

        let arg_exprs: Vec<_> = self
            .args
            .iter()
            .map(|arg| arg.argument_expression())
            .collect();

        let mut block = quote! {
            for event in #event_ident.iter() {
                #filter_check
                self.#method_ident(event #(, #arg_exprs)*).await;
            }
        };

        for arg in self.args.iter().rev() {
            block = arg.wrap_block(block);
        }

        quote! {
            if !#event_ident.is_empty() {
                #block
            }
        }
    }
}

struct HandlerArg {
    ident: Ident,
    kind: HandlerArgKind,
}

impl HandlerArg {
    fn argument_expression(&self) -> proc_macro2::TokenStream {
        let ident = &self.ident;
        match &self.kind {
            HandlerArgKind::State { mutable, .. } => {
                if *mutable {
                    quote! { &mut *#ident }
                } else {
                    quote! { &*#ident }
                }
            }
            HandlerArgKind::Service { .. } => quote! { #ident },
        }
    }

    fn wrap_block(&self, block: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
        let ident = &self.ident;
        match &self.kind {
            HandlerArgKind::State { ty, mutable } => {
                if *mutable {
                    quote! {
                        if let Some(mut #ident) = resources.get_mut::<#ty>().await {
                            #block
                        }
                    }
                } else {
                    quote! {
                        if let Some(#ident) = resources.get::<#ty>().await {
                            #block
                        }
                    }
                }
            }
            HandlerArgKind::Service { ty, service_name } => {
                let name = service_name.as_str();
                quote! {
                    if let Some(#ident) = services.get_as::<#ty>(#name) {
                        #block
                    }
                }
            }
        }
    }
}

enum HandlerArgKind {
    State { ty: Type, mutable: bool },
    Service { ty: Type, service_name: String },
}

struct SubscribeAttr {
    event_type: Type,
    filter: Option<Ident>,
}

fn parse_subscribe_attr(attr: Attribute) -> Result<SubscribeAttr> {
    attr.parse_args_with(|input: ParseStream| {
        let event_type: Type = input.parse()?;
        let mut filter = None;

        while input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
            let key: Ident = input.parse()?;
            if key == "filter" {
                input.parse::<Token![=]>()?;
                let lit: LitStr = input.parse()?;
                let ident = Ident::new(&lit.value(), lit.span());
                filter = Some(ident);
            } else {
                return Err(syn::Error::new(key.span(), "unknown #[subscribe] option"));
            }
        }

        Ok(SubscribeAttr { event_type, filter })
    })
}

fn parse_handler_arg(
    pat_type: &mut PatType,
    default_state: Option<&DefaultState>,
) -> Result<HandlerArg> {
    let ident = extract_ident(&pat_type.pat)?;
    let mut kind = None;
    let attrs = mem::take(&mut pat_type.attrs);

    for attr in attrs {
        if attr.path().is_ident("state") {
            if !matches!(attr.meta, Meta::Path(_)) {
                return Err(syn::Error::new(
                    attr.span(),
                    "#[state] does not take arguments",
                ));
            }
            kind = Some(create_state_arg(&pat_type.ty, attr.span())?);
        } else if attr.path().is_ident("service") {
            let service_name = parse_service_attr(&attr)?;
            kind = Some(create_service_arg(&pat_type.ty, service_name, attr.span())?);
        } else {
            pat_type.attrs.push(attr);
        }
    }

    if let Some(kind) = kind {
        return Ok(HandlerArg { ident, kind });
    }

    if let Some(default_state) = default_state {
        if state_matches(&pat_type.ty, default_state)? {
            return Ok(HandlerArg {
                ident,
                kind: HandlerArgKind::State {
                    ty: default_state.ty.clone(),
                    mutable: true,
                },
            });
        }
    }

    Err(syn::Error::new(
        pat_type.ty.span(),
        "additional parameters must be marked with #[state] or #[service]",
    ))
}

fn create_state_arg(ty: &Type, span: Span) -> Result<HandlerArgKind> {
    if let Type::Reference(reference) = ty {
        if reference.mutability.is_none() {
            return Err(syn::Error::new(span, "state parameters must be `&mut T`"));
        }
        Ok(HandlerArgKind::State {
            ty: reference.elem.as_ref().clone(),
            mutable: true,
        })
    } else {
        Err(syn::Error::new(span, "state parameters must be references"))
    }
}

fn create_service_arg(ty: &Type, service_name: String, span: Span) -> Result<HandlerArgKind> {
    if let Type::Reference(reference) = ty {
        if reference.mutability.is_some() {
            return Err(syn::Error::new(
                span,
                "services must be borrowed immutably as `&T`",
            ));
        }
        Ok(HandlerArgKind::Service {
            ty: reference.elem.as_ref().clone(),
            service_name,
        })
    } else {
        Err(syn::Error::new(
            span,
            "service parameters must be references",
        ))
    }
}

fn state_matches(param_ty: &Type, default: &DefaultState) -> Result<bool> {
    if let Type::Reference(reference) = param_ty {
        if reference.mutability.is_none() {
            return Err(syn::Error::new(
                reference.and_token.span(),
                "default state parameter must be `&mut` reference",
            ));
        }
        let repr = type_to_key(reference.elem.as_ref());
        Ok(repr == default.repr)
    } else {
        Err(syn::Error::new(
            param_ty.span(),
            "default state parameter must be a reference",
        ))
    }
}

fn extract_ident(pat: &Pat) -> Result<Ident> {
    if let Pat::Ident(PatIdent { ident, .. }) = pat {
        Ok(ident.clone())
    } else {
        Err(syn::Error::new(
            pat.span(),
            "parameters must be simple identifiers",
        ))
    }
}

fn parse_service_attr(attr: &Attribute) -> Result<String> {
    if matches!(attr.meta, Meta::Path(_)) {
        return Err(syn::Error::new(
            attr.span(),
            "#[service] requires a `name = \"...\"` argument",
        ));
    }

    attr.parse_args_with(|input: ParseStream| {
        if input.peek(LitStr) {
            Ok(input.parse::<LitStr>()?.value())
        } else {
            let ident: Ident = input.parse()?;
            if ident == "name" {
                input.parse::<Token![=]>()?;
                Ok(input.parse::<LitStr>()?.value())
            } else {
                Err(syn::Error::new(
                    ident.span(),
                    "expected `name = \"...\"` for #[service]",
                ))
            }
        }
    })
}

fn parse_type_value(input: ParseStream) -> Result<Type> {
    if input.peek(LitStr) {
        let lit: LitStr = input.parse()?;
        lit.parse()
    } else {
        input.parse()
    }
}

fn type_to_key(ty: &Type) -> String {
    ty.to_token_stream().to_string().replace(' ', "")
}

fn sanitize_ident(raw: &str) -> String {
    raw.chars()
        .map(|ch| match ch {
            '<' | '>' | ':' | ',' | '&' | '*' | '(' | ')' | '[' | ']' | '{' | '}' => '_',
            other if other.is_whitespace() => '_',
            other => other,
        })
        .collect()
}

/// Function-like macro that declares ISSUN events with common derives.
///
/// Generates a struct definition for each event along with the required derives
/// and an implementation of [`issun::event::Event`].
#[proc_macro]
pub fn event(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as EventMacroInput);
    let crate_name = get_crate_name();

    let expanded = input
        .events
        .into_iter()
        .map(|event| event.expand(&crate_name));

    TokenStream::from(quote! {
        #(#expanded)*
    })
}

struct EventMacroInput {
    events: Vec<EventDefinition>,
}

impl Parse for EventMacroInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut events = Vec::new();
        while !input.is_empty() {
            events.push(input.parse()?);

            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }
        Ok(Self { events })
    }
}

struct EventDefinition {
    attrs: Vec<Attribute>,
    additional_derives: Vec<Path>,
    visibility: Visibility,
    name: Ident,
    fields: EventFields,
}

impl EventDefinition {
    fn expand(self, crate_name: &proc_macro2::TokenStream) -> proc_macro2::TokenStream {
        let mut derives = vec![
            syn::parse_quote!(Debug),
            syn::parse_quote!(Clone),
            syn::parse_quote!(::serde::Serialize),
            syn::parse_quote!(::serde::Deserialize),
        ];

        for path in self.additional_derives {
            push_unique_path(&mut derives, path);
        }

        let derive_attr = quote! {
            #[derive(#(#derives),*)]
        };

        let body = match self.fields {
            EventFields::Unit => quote!(;),
            EventFields::Struct(fields) => {
                let rendered_fields = fields.into_iter().map(|field| {
                    let attrs = field.attrs;
                    let vis = field.visibility;
                    let name = field.name;
                    let ty = field.ty;
                    quote! {
                        #(#attrs)*
                        #vis #name: #ty
                    }
                });

                quote! {
                    {
                        #(#rendered_fields,)*
                    }
                }
            }
        };

        let attrs = self.attrs;
        let vis = self.visibility;
        let name = self.name;

        quote! {
            #(#attrs)*
            #derive_attr
            #vis struct #name #body

            impl #crate_name::event::Event for #name {}
        }
    }
}

impl Parse for EventDefinition {
    fn parse(input: ParseStream) -> Result<Self> {
        let outer_attrs = input.call(Attribute::parse_outer)?;
        let mut attrs = Vec::new();
        let mut derives = Vec::new();

        for attr in outer_attrs {
            if attr.path().is_ident("derive") {
                let parsed: Punctuated<Path, Token![,]> =
                    attr.parse_args_with(Punctuated::parse_terminated)?;
                derives.extend(parsed.into_iter());
            } else {
                attrs.push(attr);
            }
        }

        let visibility = if input.peek(Token![pub]) {
            input.parse()?
        } else {
            Visibility::Inherited
        };

        if input.peek(Token![struct]) {
            input.parse::<Token![struct]>()?;
        }

        let name: Ident = input.parse()?;

        let fields = if input.peek(Token![;]) {
            input.parse::<Token![;]>()?;
            EventFields::Unit
        } else {
            let content;
            braced!(content in input);

            let mut parsed_fields = Vec::new();
            while !content.is_empty() {
                let attrs = content.call(Attribute::parse_outer)?;
                if content.is_empty() {
                    return Err(content.error("expected field definition after attributes"));
                }

                let visibility = if content.peek(Token![pub]) {
                    content.parse()?
                } else {
                    Visibility::Inherited
                };
                let name: Ident = content.parse()?;
                content.parse::<Token![:]>()?;
                let ty: Type = content.parse()?;

                if content.peek(Token![,]) {
                    content.parse::<Token![,]>()?;
                } else if content.peek(Token![;]) {
                    content.parse::<Token![;]>()?;
                } else if !content.is_empty() {
                    return Err(content.error("expected `,` or `;` after field"));
                }

                parsed_fields.push(EventField {
                    attrs,
                    visibility,
                    name,
                    ty,
                });
            }

            EventFields::Struct(parsed_fields)
        };

        if input.peek(Token![;]) {
            input.parse::<Token![;]>()?;
        }

        Ok(Self {
            attrs,
            additional_derives: derives,
            visibility,
            name,
            fields,
        })
    }
}

enum EventFields {
    Unit,
    Struct(Vec<EventField>),
}

struct EventField {
    attrs: Vec<Attribute>,
    visibility: Visibility,
    name: Ident,
    ty: Type,
}

fn push_unique_path(paths: &mut Vec<Path>, new_path: Path) {
    let repr = path_to_string(&new_path);
    if paths
        .iter()
        .all(|existing| path_to_string(existing) != repr)
    {
        paths.push(new_path);
    }
}

fn path_to_string(path: &Path) -> String {
    path.to_token_stream().to_string()
}

/// Attribute macro that injects `pump_event_systems` calls before/after input handlers.
#[proc_macro_attribute]
pub fn auto_pump(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as AutoPumpArgs);

    let item_clone = item.clone();
    if let Ok(mut function) = syn::parse::<ItemFn>(item_clone) {
        match apply_auto_pump(&function.sig, &mut function.block, &args) {
            Ok(()) => return TokenStream::from(quote! { #function }),
            Err(err) => return err.to_compile_error().into(),
        }
    }

    let mut method = parse_macro_input!(item as ImplItemFn);
    match apply_auto_pump(&method.sig, &mut method.block, &args) {
        Ok(()) => TokenStream::from(quote! { #method }),
        Err(err) => err.to_compile_error().into(),
    }
}

#[derive(Clone)]
struct AutoPumpArgs {
    before: bool,
    after: bool,
    pump_fn: Option<Path>,
    sides_specified: bool,
}

impl Default for AutoPumpArgs {
    fn default() -> Self {
        Self {
            before: true,
            after: true,
            pump_fn: None,
            sides_specified: false,
        }
    }
}

impl Parse for AutoPumpArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.is_empty() {
            return Ok(Self::default());
        }

        let mut args = AutoPumpArgs {
            before: false,
            after: false,
            pump_fn: None,
            sides_specified: false,
        };

        while !input.is_empty() {
            let ident: Ident = input.parse()?;
            match ident.to_string().as_str() {
                "before" => {
                    args.before = true;
                    args.sides_specified = true;
                }
                "after" => {
                    args.after = true;
                    args.sides_specified = true;
                }
                "pump_fn" => {
                    input.parse::<Token![=]>()?;
                    let path = if input.peek(LitStr) {
                        input.parse::<LitStr>()?.parse::<Path>()?
                    } else {
                        input.parse::<Path>()?
                    };
                    args.pump_fn = Some(path);
                }
                other => {
                    return Err(syn::Error::new(
                        ident.span(),
                        format!("Unknown auto_pump option `{}`", other),
                    ));
                }
            }

            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        if !args.sides_specified {
            args.before = true;
            args.after = true;
        }

        Ok(args)
    }
}

struct PumpParams {
    services: Ident,
    systems: Ident,
    resources: Ident,
}

fn apply_auto_pump(signature: &Signature, block: &mut Block, args: &AutoPumpArgs) -> Result<()> {
    if !args.before && !args.after {
        return Ok(());
    }

    let params = extract_pump_params(signature)?;
    let pump_path = args.pump_fn.clone().unwrap_or_else(|| {
        syn::parse_str::<Path>("crate::plugins::pump_event_systems").expect("valid path")
    });

    let mut original = mem::take(&mut block.stmts);
    let mut stmts = Vec::new();

    // Add 'before' pump call
    if args.before {
        stmts.push(build_pump_stmt(&pump_path, &params));
    }

    // Check if the last statement is a trailing expression (no semicolon)
    // If so, we need to insert 'after' pump BEFORE it to preserve return value
    let has_trailing_expr = original
        .last()
        .map_or(false, |stmt| matches!(stmt, Stmt::Expr(_, None)));

    if args.after && has_trailing_expr {
        // Insert all but last statement
        if original.len() > 1 {
            stmts.extend(original.drain(..original.len() - 1));
        }
        // Insert 'after' pump
        stmts.push(build_pump_stmt(&pump_path, &params));
        // Insert trailing expression last
        stmts.extend(original);
    } else {
        // No trailing expression, just append everything
        stmts.extend(original);
        if args.after {
            stmts.push(build_pump_stmt(&pump_path, &params));
        }
    }

    block.stmts = stmts;
    Ok(())
}

fn build_pump_stmt(path: &Path, params: &PumpParams) -> Stmt {
    let services = &params.services;
    let systems = &params.systems;
    let resources = &params.resources;

    // Use quote! to generate tokens, then parse into Stmt
    let tokens = quote! {
        #path(#services, #systems, #resources).await;
    };
    syn::parse2(tokens).expect("failed to parse pump statement")
}

fn extract_pump_params(signature: &Signature) -> Result<PumpParams> {
    let mut services = None;
    let mut systems = None;
    let mut resources = None;

    for input in signature.inputs.iter() {
        if let FnArg::Typed(pat_type) = input {
            if let Ok(ident) = extract_ident(&pat_type.pat) {
                if matches_context(&pat_type.ty, "ServiceContext") {
                    services = Some(ident);
                } else if matches_context(&pat_type.ty, "SystemContext") {
                    systems = Some(ident);
                } else if matches_context(&pat_type.ty, "ResourceContext") {
                    resources = Some(ident);
                }
            }
        }
    }

    match (services, systems, resources) {
        (Some(sv), Some(sys), Some(res)) => Ok(PumpParams {
            services: sv,
            systems: sys,
            resources: res,
        }),
        _ => Err(syn::Error::new(
            signature.fn_token.span(),
            "#[auto_pump] requires parameters for ServiceContext, SystemContext, and ResourceContext",
        )),
    }
}

fn matches_context(ty: &Type, expected: &str) -> bool {
    match ty {
        Type::Reference(reference) => match reference.elem.as_ref() {
            Type::Path(path) => path
                .path
                .segments
                .last()
                .map(|segment| segment.ident == expected)
                .unwrap_or(false),
            _ => false,
        },
        Type::Path(path) => path
            .path
            .segments
            .last()
            .map(|segment| segment.ident == expected)
            .unwrap_or(false),
        _ => false,
    }
}
