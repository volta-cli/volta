// The `proc_macro2` crate is a polyfill for advanced functionality of Rust's
// procedural macros, not all of which have shipped in stable Rust. It's used by
// the `syn` and `quote` crates to produce a shimmed version of the standard
// `TokenStream` type. So internally that's the type we have to use for the
// implementation of our macro. The actual front-end for the macro takes this
// shimmed `TokenStream` type and converts it to the built-in `TokenStream` type
// required by the Rust macro system.
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Attribute, Ident, LitStr, Visibility};

// These seem to be leaked implementation details of the `quote` macro that have
// to be imported by users. You can ignore them; they simply pacify the compiler.
#[allow(unused_imports)]
use quote::{pounded_var_names, quote_each_token, quote_spanned};

/// The intermediate representation (IR) of a struct type defined by the `layout!`
/// macro, which contains the flattened directory entries, organized into three
/// categories:
///
/// - Directories
/// - Executable files
/// - Other files
pub(crate) struct Ir {
    pub(crate) name: Ident,
    pub(crate) attrs: Vec<Attribute>,
    pub(crate) visibility: Visibility,
    pub(crate) dirs: Vec<Entry>,
    pub(crate) files: Vec<Entry>,
    pub(crate) exes: Vec<Entry>,
}

impl Ir {
    fn dir_names(&self) -> impl Iterator<Item = &Ident> {
        self.dirs.iter().map(|entry| &entry.name)
    }

    fn file_names(&self) -> impl Iterator<Item = &Ident> {
        self.files.iter().map(|entry| &entry.name)
    }

    fn exe_names(&self) -> impl Iterator<Item = &Ident> {
        self.exes.iter().map(|entry| &entry.name)
    }

    fn field_names(&self) -> impl Iterator<Item = &Ident> {
        let dir_names = self.dir_names();
        let file_names = self.file_names();
        let exe_names = self.exe_names();
        dir_names.chain(file_names).chain(exe_names)
    }

    fn to_struct_decl(&self) -> TokenStream {
        let name = &self.name;

        let attrs = self.attrs.iter();
        let visibility = self.visibility.clone();

        let field_names = self.field_names().map(|field_name| {
            // Use the field name's span for good duplicate-field-name error messages.
            quote_spanned! {field_name.span()=>
                #field_name : ::std::path::PathBuf ,
            }
        });

        quote! {
            #(#attrs)* #visibility struct #name {
                #(#field_names)*
                root: ::std::path::PathBuf,
            }
        }
    }

    fn to_create_method(&self) -> TokenStream {
        let name = &self.name;
        let dir_names = self.dir_names();

        quote! {
            impl #name {
                /// Creates all subdirectories in this directory layout.
                pub fn create(&self) -> ::std::io::Result<()> {
                    #(::std::fs::create_dir_all(self.#dir_names())?;)*
                    ::std::result::Result::Ok(())
                }
            }
        }
    }

    fn to_item_methods(&self) -> TokenStream {
        let name = &self.name;

        let methods = self.field_names().map(|field_name| {
            // Markdown-formatted field name for the doc comment.
            let markdown_field_name = format!("`{}`", field_name);
            let markdown_field_name = LitStr::new(&markdown_field_name, field_name.span());

            // Use the field name's span for good duplicate-method-name error messages.
            quote_spanned! {field_name.span()=>
                #[doc = "Returns the "]
                #[doc = #markdown_field_name]
                #[doc = " path."]
                pub fn #field_name(&self) -> &::std::path::Path { &self.#field_name }
            }
        });

        quote! {
            impl #name {
                #(#methods)*

                 /// Returns the root path for this directory layout.
                pub fn root(&self) -> &::std::path::Path { &self.root }
            }
        }
    }

    fn to_ctor(&self) -> TokenStream {
        let name = &self.name;
        let root = Ident::new("root", self.name.span());

        let dir_names = self.dir_names();
        let dir_inits = self.dirs.iter().map(|entry| entry.to_normal_init(&root));

        let file_names = self.file_names();
        let file_inits = self.files.iter().map(|entry| entry.to_normal_init(&root));

        let exe_names = self.exe_names();
        let exe_inits = self.exes.iter().map(|entry| entry.to_exe_init(&root));

        let all_names = dir_names.chain(file_names).chain(exe_names);
        let all_inits = dir_inits.chain(file_inits).chain(exe_inits);

        let markdown_struct_name = format!("`{}`", name);
        let markdown_struct_name = LitStr::new(&markdown_struct_name, name.span());

        quote! {
            impl #name {
                #[doc = "Constructs a new instance of the "]
                #[doc = #markdown_struct_name]
                #[doc = " layout, rooted at `root`."]
                pub fn new(#root: ::std::path::PathBuf) -> Self {
                    Self {
                        #(#all_names: #all_inits),* ,
                        #root: #root
                    }
                }
            }
        }
    }

    pub(crate) fn codegen(&self) -> TokenStream {
        let struct_decl = self.to_struct_decl();
        let ctor = self.to_ctor();
        let item_methods = self.to_item_methods();
        let create_method = self.to_create_method();

        quote! {
            #struct_decl
            #ctor
            #item_methods
            #create_method
        }
    }
}

pub(crate) struct Entry {
    pub(crate) name: Ident,
    pub(crate) context: Vec<LitStr>,
    pub(crate) filename: LitStr,
}

impl Entry {
    fn to_normal_init(&self, root: &Ident) -> TokenStream {
        let name = &self.name;
        let path_items = self.context.iter();
        let name_replicated = self.context.iter().map(|_| name);
        let filename = &self.filename;

        quote! {
            {
                let mut #name = #root.clone();
                #(#name_replicated.push(#path_items);)*
                #name.push(#filename);
                #name
            }
        }
    }

    fn to_exe_init(&self, root: &Ident) -> TokenStream {
        let name = &self.name;
        let path_items = self.context.iter();
        let name_replicated = self.context.iter().map(|_| name);
        let filename = &self.filename;

        quote! {
            {
                let mut #name = #root.clone();
                #(#name_replicated.push(#path_items);)*
                #name.push(::std::format!("{}{}", #filename, ::std::env::consts::EXE_SUFFIX));
                #name
            }
        }
    }
}
