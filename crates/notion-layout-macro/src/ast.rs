use crate::ir::{Ir, Entry};
use syn::parse::{Parse, ParseStream, Result};
use syn::punctuated::Punctuated;
use syn::{braced, token, Attribute, Ident, LitStr, Token, Visibility};

/// Abstract syntax tree (AST) for the surface syntax of the `layout!` macro.
///
/// The surface syntax of the `layout!` macro takes the form:
///
/// ```
/// Attribute* Visibility "struct" Ident Directory
/// ```
///
/// This AST gets lowered by the `flatten` method to a flat list of entries,
/// organized by entry type. See the `Ir` type for details.
pub(crate) struct Ast {
    attrs: Vec<Attribute>,
    visibility: Visibility,
    name: Ident,
    directory: Directory,
}

impl Parse for Ast {
    fn parse(input: ParseStream) -> Result<Self> {
        let attrs: Vec<Attribute> = input.call(Attribute::parse_outer)?;
        let visibility: Visibility = input.parse()?;
        input.parse::<Token![struct]>()?;
        let name: Ident = input.parse()?;
        let directory: Directory = input.parse()?;
        Ok(Ast { attrs, visibility, name, directory })
    }
}

impl Ast {
    /// Lowers the AST to a flattened intermediate representation.
    pub(crate) fn flatten(self) -> Ir {
        let mut results = Ir {
            name: self.name,
            attrs: self.attrs,
            visibility: self.visibility,
            dirs: vec![],
            files: vec![],
            exes: vec![],
        };

        self.directory.flatten(&mut results, vec![]);

        results
    }
}

/// Represents a directory entry in the AST, which can recursively contain
/// more entries.
///
/// The surface syntax of a directory takes the form:
///
/// ```
/// {
///     (FieldPrefix)FieldContents*
/// }
/// ```
struct Directory {
    brace_token: token::Brace,
    entries: Punctuated<FieldPrefix, FieldContents>,
}

impl Parse for Directory {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        Ok(Directory {
            brace_token: braced!(content in input),
            entries: content.parse_terminated(FieldPrefix::parse)?,
        })
    }
}

// FIXME: this should have proper validation for no dupes, with good spans for error messages

impl Directory {
    /// Lowers the directory to a flattened intermediate representation.
    fn flatten(self, results: &mut Ir, context: Vec<LitStr>) {
        for pair in self.entries.into_pairs() {
            let (prefix, punc) = pair.into_tuple();

            let mut entry = Entry {
                name: prefix.name,
                context: context.clone(),
                filename: prefix.filename.clone(),
            };

            match punc {
                Some(FieldContents::Dir(dir)) => {
                    results.dirs.push(entry);
                    let mut sub_context = context.clone();
                    sub_context.push(prefix.filename);
                    dir.flatten(results, sub_context);
                }
                _ => {
                    let filename = prefix.filename.value();
                    if filename.ends_with(".exe") {
                        let filename = &filename[0..filename.len() - 4];
                        entry.filename = LitStr::new(filename, prefix.filename.span());
                        results.exes.push(entry);
                    } else {
                        results.files.push(entry);
                    }
                }
            }
        }
    }
}

/// AST for the common prefix of a single field in a `layout!` struct declaration,
/// which is of the form:
///
/// ```
/// LitStr ":" Ident
/// ```
///
/// This is followed either by a semicolon (`;`), indicating that the field is a
/// file, or a braced directory entry, indicating that the field is a directory.
///
/// If the `LitStr` contains the suffix `".exe"`, it is treated specially as an
/// executable file, whose suffix (or lack thereof) is determined by the current
/// operating system (using the `std::env::consts::EXE_SUFFIX` constant).
struct FieldPrefix {
    filename: LitStr,
    colon: Token![:],
    name: Ident,
}

impl Parse for FieldPrefix {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(FieldPrefix {
            filename: input.parse()?,
            colon: input.parse()?,
            name: input.parse()?,
        })
    }
}

/// AST for the suffix of a field in a `layout!` struct declaration.
enum FieldContents {
    /// A file field suffix, which consists of a single semicolon (`;`).
    File(Token![;]),

    /// A directory field suffix, which consists of a braced directory.
    Dir(Directory),
}

impl Parse for FieldContents {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        Ok(if lookahead.peek(Token![;]) {
            let semi = input.parse()?;
            FieldContents::File(semi)
        } else {
            let directory = input.parse()?;
            FieldContents::Dir(directory)
        })
    }
}
