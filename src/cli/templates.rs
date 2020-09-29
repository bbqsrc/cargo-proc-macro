// SPDX-License-Identifier: EUPL-1.2+

pub const ATTR_BASE_TMPL: &str = "extern crate proc_macro;

use proc_macro::TokenStream;
use syn::parse_macro_input;

#[proc_macro_attribute]
pub fn @SNAKE_NAME@(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr = parse_macro_input!(attr as proc_macro2::TokenStream);
    let item = parse_macro_input!(item as proc_macro2::TokenStream);

    match @SNAKE_NAME@_macro::@SNAKE_NAME@(attr, item) {
        Ok(tokens) => tokens.into(),
        Err(err) => TokenStream::from(err.to_compile_error()),
    }
}
";

pub const ATTR_CRATE_TMPL: &str = "use proc_macro2::TokenStream;
use quote::quote;

pub fn @SNAKE_NAME@(attr: TokenStream, item: TokenStream) -> Result<TokenStream, syn::Error> {
    // Implement your proc-macro logic here. :)
    Ok(item)
}
";

pub const ATTR_WKSP_MSG: &str = "-- Created workspace with `@NAME@` and `@NAME@_macro` crates.

`@NAME@` is the crate you should use in Rust projects. For example:

    use @SNAKE_NAME@::@SNAKE_NAME@;
    #[@SNAKE_NAME@]
    fn some_compatible_element() { ... }

The testable logic for your macro lives in `@NAME@_macro` and is a dependency of `@NAME@`.";


pub const DERIVE_BASE_TMPL: &str = "extern crate proc_macro;

use proc_macro::TokenStream;
use syn::parse_macro_input;

#[proc_macro_derive(@NAME@)]
pub fn derive_@SNAKE_NAME@(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr = parse_macro_input!(attr as proc_macro2::TokenStream);
    let item = parse_macro_input!(item as proc_macro2::TokenStream);

    match @SNAKE_NAME@_macro::derive_@SNAKE_NAME@(attr, item) {
        Ok(tokens) => tokens.into(),
        Err(err) => TokenStream::from(err.to_compile_error()),
    }
}
";

pub const DERIVE_CRATE_TMPL: &str = "use proc_macro2::TokenStream;
use quote::quote;

pub fn derive_@SNAKE_NAME@(item: TokenStream) -> Result<TokenStream, syn::Error> {
    // Implement your proc-macro logic here. :)
    Ok(item)
}
";

pub const DERIVE_WKSP_MSG: &str = "-- Created workspace with `@NAME@` and `@NAME@_macro` crates.

`@NAME@` is the crate you should use in Rust projects. For example:

    use @SNAKE_NAME@::@SNAKE_NAME@;
    #[derive(@NAME@)]
    struct SomeStruct;

The testable logic for your macro lives in `@NAME@_macro` and is a dependency of `@NAME@`.";

pub const FUNCTION_BASE_TMPL: &str = "extern crate proc_macro;

use proc_macro::TokenStream;
use syn::parse_macro_input;

#[proc_macro]
pub fn @SNAKE_NAME@(item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as proc_macro2::TokenStream);

    match @SNAKE_NAME@_macro::@SNAKE_NAME@(item) {
        Ok(tokens) => tokens.into(),
        Err(err) => TokenStream::from(err.to_compile_error()),
    }
}
";

pub const FUNCTION_CRATE_TMPL: &str = "use proc_macro2::TokenStream;
use quote::quote;

pub fn @SNAKE_NAME@(item: TokenStream) -> Result<TokenStream, syn::Error> {
    // Implement your proc-macro logic here. :)
    Ok(item)
}
";

pub const FUNCTION_WKSP_MSG: &str = "-- Created workspace with `@NAME@` and `@NAME@_macro` crates.

`@NAME@` is the crate you should use in Rust projects. For example:

    use @SNAKE_NAME@::@SNAKE_NAME@;
    fn some_fn() {
        @SNAKE_NAME@!(...);
    }

The testable logic for your macro lives in `@NAME@_macro` and is a dependency of `@NAME@`.";
