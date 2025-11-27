//! log! macro for simplified EventLog access
//!
//! # Example
//! ```ignore
//! // Before:
//! app.world_mut().resource_mut::<EventLog>()
//!     .add(format!("✅ {} quarantined", city.name));
//!
//! // After:
//! log!(app, "✅ {} quarantined", city.name);
//! ```

use proc_macro::TokenStream;
use quote::quote;

/// Proc macro for simplified EventLog::add() calls
pub fn log_impl(input: TokenStream) -> TokenStream {
    let input_str = input.to_string();

    // Parse the macro input: first argument is the world/app, rest is format string + args
    let parts: Vec<&str> = input_str.splitn(2, ',').collect();

    if parts.len() < 2 {
        return syn::Error::new(
            proc_macro2::Span::call_site(),
            "log! macro requires at least 2 arguments: log!(app, \"message\", ...args)",
        )
        .to_compile_error()
        .into();
    }

    let world_expr: proc_macro2::TokenStream = parts[0].trim().parse().unwrap();
    let message_and_args: proc_macro2::TokenStream = parts[1].trim().parse().unwrap();

    let expanded = quote! {
        #world_expr.world_mut().resource_mut::<EventLog>()
            .add(format!(#message_and_args))
    };

    TokenStream::from(expanded)
}
