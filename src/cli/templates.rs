// SPDX-License-Identifier: EUPL-1.2+

pub const ATTR_BASE_TMPL: &str = "extern crate proc_macro;

use proc_macro::TokenStream;
use syn::parse_macro_input;

#[proc_macro_attribute]
pub fn @NAME@(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr = parse_macro_input!(attr as proc_macro2::TokenStream);
    let item = parse_macro_input!(item as proc_macro2::TokenStream);

    match @NAME@_macro::@NAME@(attr, item) {
        Ok(tokens) => tokens.into(),
        Err(err) => TokenStream::from(err.to_compile_error()),
    }
}
";

pub const ATTR_CRATE_TMPL: &str = "use proc_macro2::TokenStream;
use quote::quote;

pub fn @NAME@(attr: TokenStream, item: TokenStream) -> Result<TokenStream, syn::Error> {
    // Implement your proc-macro logic here. :)
    Ok(item)
}
";

pub const DERIVE_BASE_TMPL: &str = "extern crate proc_macro;

use proc_macro::TokenStream;
use syn::parse_macro_input;

#[proc_macro_derive(@NAME@)]
pub fn derive_@NAME@(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr = parse_macro_input!(attr as proc_macro2::TokenStream);
    let item = parse_macro_input!(item as proc_macro2::TokenStream);

    match @NAME@_macro::derive_@NAME@(attr, item) {
        Ok(tokens) => tokens.into(),
        Err(err) => TokenStream::from(err.to_compile_error()),
    }
}
";

pub const DERIVE_CRATE_TMPL: &str = "use proc_macro2::TokenStream;
use quote::quote;

pub fn derive_@NAME@(item: TokenStream) -> Result<TokenStream, syn::Error> {
    // Implement your proc-macro logic here. :)
    Ok(item)
}
";

pub const FUNCTION_BASE_TMPL: &str = "extern crate proc_macro;

use proc_macro::TokenStream;
use syn::parse_macro_input;

#[proc_macro]
pub fn @NAME@(item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as proc_macro2::TokenStream);

    match @NAME@_macro::@NAME@(item) {
        Ok(tokens) => tokens.into(),
        Err(err) => TokenStream::from(err.to_compile_error()),
    }
}
";

pub const FUNCTION_CRATE_TMPL: &str = "use proc_macro2::TokenStream;
use quote::quote;

pub fn @NAME@(item: TokenStream) -> Result<TokenStream, syn::Error> {
    // Implement your proc-macro logic here. :)
    Ok(item)
}
";
