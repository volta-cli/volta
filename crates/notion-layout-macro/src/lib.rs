#![recursion_limit = "128"]

extern crate proc_macro;

mod ast;
mod ir;

use crate::ast::Ast;
use proc_macro::TokenStream;
use syn::parse_macro_input;

/// A macro for defining Notion directory layout hierarchies.
///
/// The syntax of `layout!` takes the form:
///
/// ```
/// layout! {
///     LayoutStruct*
/// }
/// ```
///
/// The syntax of a `LayoutStruct` takes the form:
///
/// ```
/// Attribute* Visibility "struct" Ident Directory
/// ```
///
/// The syntax of a `Directory` takes the form:
///
/// ```
/// {
///     (FieldPrefix)FieldContents*
/// }
/// ```
///
/// The syntax of a `FieldPrefix` takes the form:
///
/// ```
/// LitStr ":" Ident
/// ```
///
/// The syntax of a `FieldContents` is either:
///
/// ```
/// ";"
/// ```
///
/// or:
///
/// ```
/// Directory
/// ```
#[proc_macro]
pub fn layout(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as Ast);
    let expanded = ast.compile();
    TokenStream::from(expanded)
}
