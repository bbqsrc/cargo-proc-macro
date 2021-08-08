// SPDX-License-Identifier: EUPL-1.2+

pub const WKSP_MSG_HEADER: &str =
    "`@NAME@` is the crate you should use in Rust projects. For example:";

pub const WKSP_MSG_FOOTER: &str =
    "The testable logic for your macro lives in `impl`. The proc-macro itself is
implemented in the `macro` directory, and is a dependency of `@NAME@`.";

pub const WKSP_ATTR_EXAMPLE: &str = "
use @SNAKE_NAME@::@SNAKE_NAME@;

#[@SNAKE_NAME@]
fn some_compatible_element() { ... }
";

pub const WKSP_DERIVE_EXAMPLE: &str = "
use @SNAKE_NAME@::@STRUCT_NAME@;

#[derive(@STRUCT_NAME@)]
struct SomeStruct;
";

pub const WKSP_FUNCTION_EXAMPLE: &str = "
use @SNAKE_NAME@::@SNAKE_NAME@;

fn some_fn() {
    @SNAKE_NAME@!(...);
}
";

pub const ATTR_LIB_TMPL: &str = "#[doc(inline)]
pub use @SNAKE_NAME@_macro::@SNAKE_NAME@;
";

pub const ATTR_MACRO_TMPL: &str =
    "//! This crate implements the macro for `@SNAKE_NAME@` and should not be used directly.
extern crate proc_macro;

use proc_macro::TokenStream;
use syn::parse_macro_input;

#[proc_macro_attribute]
/// Document your macro here.
pub fn @SNAKE_NAME@(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr = parse_macro_input!(attr as proc_macro2::TokenStream);
    let item = parse_macro_input!(item as proc_macro2::TokenStream);

    match @SNAKE_NAME@_impl::@SNAKE_NAME@(attr, item) {
        Ok(tokens) => tokens.into(),
        Err(err) => TokenStream::from(err.to_compile_error()),
    }
}
";

pub const ATTR_IMPL_TMPL: &str =
    "//! This crate implements the macro for `@SNAKE_NAME@` and should not be used directly.

use proc_macro2::TokenStream;
use quote::quote;

#[doc(hidden)]
pub fn @SNAKE_NAME@(_attr: TokenStream, _item: TokenStream) -> Result<TokenStream, syn::Error> {
    // Implement your proc-macro logic here. :)
    Ok(quote! {
        \"Hello world!\"
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn example() {
        assert!(@SNAKE_NAME@(quote! {}, quote! {}).is_ok());
    }
}
";

pub const DERIVE_LIB_TMPL: &str = "#[doc(inline)]
pub use @SNAKE_NAME@_macro::@STRUCT_NAME@;
";

pub const DERIVE_MACRO_TMPL: &str =
    "//! This crate implements the macro for `@SNAKE_NAME@` and should not be used directly.

extern crate proc_macro;

use proc_macro::TokenStream;
use syn::parse_macro_input;

#[proc_macro_derive(@STRUCT_NAME@)]
/// Document your macro here.
pub fn derive_@SNAKE_NAME@(item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as proc_macro2::TokenStream);

    match @SNAKE_NAME@_impl::derive_@SNAKE_NAME@(item) {
        Ok(tokens) => tokens.into(),
        Err(err) => TokenStream::from(err.to_compile_error()),
    }
}
";

pub const DERIVE_IMPL_TMPL: &str =
    "//! This crate implements the macro for `@SNAKE_NAME@` and should not be used directly.

use proc_macro2::TokenStream;
use quote::quote;

#[doc(hidden)]
pub fn derive_@SNAKE_NAME@(_item: TokenStream) -> Result<TokenStream, syn::Error> {
    // Implement your proc-macro logic here. :)
    Ok(quote! {
        \"Hello world!\"
    })
}
";

pub const FUNCTION_LIB_TMPL: &str = "#[doc(inline)]
pub use @SNAKE_NAME@_macro::@SNAKE_NAME@;
";

pub const FUNCTION_MACRO_TMPL: &str =
    "//! This crate implements the macro for `@SNAKE_NAME@` and should not be used directly.

extern crate proc_macro;

use proc_macro::TokenStream;
use syn::parse_macro_input;

#[proc_macro]
/// Document your macro here.
pub fn @SNAKE_NAME@(item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as proc_macro2::TokenStream);

    match @SNAKE_NAME@_impl::@SNAKE_NAME@(item) {
        Ok(tokens) => tokens.into(),
        Err(err) => TokenStream::from(err.to_compile_error()),
    }
}
";

pub const FUNCTION_IMPL_TMPL: &str =
    "//! This crate implements the macro for `@SNAKE_NAME@` and should not be used directly.

use proc_macro2::TokenStream;
use quote::quote;

#[doc(hidden)]
pub fn @SNAKE_NAME@(_item: TokenStream) -> Result<TokenStream, syn::Error> {
    // Implement your proc-macro logic here. :)
    Ok(quote! {
        \"Hello world!\"
    })
}
";
