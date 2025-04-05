// SPDX-FileCopyrightText: Copyright © 2025 Serpent OS Developers
// SPDX-FileCopyrightText: Copyright © 2025 AerynOS Developers
//
// SPDX-License-Identifier: MPL-2.0

use proc_macro::TokenStream;
use proc_macro2::Span;
use syn::parse::ParseStream;
use syn::parse_macro_input;
use syn::{parse::Parse, Ident, LitStr, Result};

/// Parse the arguments of the authorized attribute
struct AuthorizedArgs {
    /// The polkit action ID
    action_id: LitStr,
    /// Optional keywords and their values
    keywords: Vec<(Ident, LitStr)>,
}

impl Parse for AuthorizedArgs {
    fn parse<'a>(input: ParseStream<'a>) -> Result<Self> {
        let action_id = input.parse::<LitStr>()?;
        let keywords = Vec::new();
        Ok(AuthorizedArgs { action_id, keywords })
    }
}

/// An attribute macro for requiring polkit authorization to access an API method
///
/// This macro wraps a method implementation with authorization checks based on
/// polkit action IDs. It extracts the auth service from `self.auth` and performs
/// the check before executing the actual method logic.
///
/// # Syntax
///
/// ```
/// #[authorized("com.aerynos.lichen.read.disks")]
/// async fn my_method(...) { ... }
/// ```
///
/// # Keyword Arguments
///
/// - `message="Custom error message"` - Custom error message (optional)
///
/// ```
#[proc_macro_attribute]
pub fn authorized(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as AuthorizedArgs);

    // Extract the action ID and error message
    let action_id = args.action_id;
    let error_message = args
        .keywords
        .iter()
        .find(|(key, _)| key == "message")
        .map(|(_, value)| value.clone())
        .unwrap_or_else(|| {
            LitStr::new(
                &format!("Not authorized for action: {}", action_id.value()),
                Span::call_site(),
            )
        });

    // Convert the item to a string
    let item_str = item.to_string();

    // Find the position after "Box::pin(async move {" or similar
    if let Some(box_pin_pos) = item_str.find("pin(") {
        if let Some(async_move_pos) = item_str[box_pin_pos..].find("{") {
            let insert_pos = box_pin_pos + async_move_pos + 1; // +1 to get past the opening brace

            // Create the authorization code as a string
            let auth_code = format!(
                r#"
                    let request = match self.auth.verify_request(request, "{}").await {{
                        Ok(req) => req,
                        Err(e) => {{
                            if e.code() == tonic::Code::PermissionDenied {{
                                return Err(tonic::Status::permission_denied("{}"));
                            }} else {{
                                return Err(e);
                            }}
                        }}
                    }};
                "#,
                action_id.value(),
                error_message.value()
            );

            // Insert the authorization code after the opening brace of the async block
            let modified_item_str = format!("{}{}{}", &item_str[..insert_pos], auth_code, &item_str[insert_pos..]);
            // Parse the modified string back to a TokenStream
            return modified_item_str.parse().unwrap();
        }
    } else {
        panic!()
    }

    // If we couldn't find the Box::pin expression, return the original item
    item
}
