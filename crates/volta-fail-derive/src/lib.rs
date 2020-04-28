// Required for now, as there is not yet support for `proc_macro` as a built-in.
extern crate proc_macro;

use quote::*;

use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use syn::Meta::{List, NameValue, Path};
use syn::{DeriveInput, Lit, NestedMeta};

#[proc_macro_derive(VoltaFail, attributes(volta_fail))]
pub fn volta_fail(token_stream: TokenStream) -> TokenStream {
    let input: DeriveInput = syn::parse(token_stream).unwrap();
    let name = &input.ident;

    let mut code = Ident::new("UnknownError", Span::call_site());
    let mut code_set = false;

    for meta in input.attrs.iter().filter_map(get_volta_fail_meta_items) {
        for item in meta {
            match item {
                NestedMeta::Lit(_) => {
                    panic!("#[volta_fail()]: must be name/value pairs, not a literal");
                }

                NestedMeta::Meta(List(_)) => {
                    panic!("#[volta_fail()]: must be name/value pairs, not a list");
                }

                NestedMeta::Meta(NameValue(ref m)) if m.path.is_ident("code") => {
                    if let Lit::Str(s) = &m.lit {
                        code = Ident::new(&s.value(), Span::call_site());
                        code_set = true;
                    } else {
                        // Defined, but not a string.
                        panic!("#[volta_fail()]: 'code' must be a string.");
                    }
                }

                NestedMeta::Meta(NameValue(m)) => {
                    panic!(
                        "#[volta_fail()]: not a recognized name: '{}'",
                        m.path.to_token_stream()
                    );
                }

                NestedMeta::Meta(Path(_)) => {
                    panic!("#[volta_fail()]: must be name/value pairs, not an identifier");
                }
            }
        }
    }

    if !code_set {
        panic!("#[volta_fail()] must set an exit code");
    }

    let tokens = quote! {
        impl VoltaFail for #name {
            fn exit_code(&self) -> ExitCode {
                ExitCode::#code
            }
        }
    };

    tokens.into()
}

fn get_volta_fail_meta_items(attr: &syn::Attribute) -> Option<Vec<NestedMeta>> {
    if attr.path.segments.len() == 1 && attr.path.segments[0].ident == "volta_fail" {
        match attr.parse_meta() {
            Ok(List(ref meta)) => Some(meta.nested.iter().cloned().collect()),

            _ => {
                panic!("#[volta_fail()] must be a list of attributes");
            }
        }
    } else {
        None
    }
}
