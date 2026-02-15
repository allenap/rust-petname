use proc_macro::TokenStream;
use std::collections::HashSet;
use std::path::PathBuf;

use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::{Ident, LitStr, Token};

/// Input to the `words!` macro.
///
/// Supports two forms:
/// - `words!("path/to/adjectives.txt")` — name derived from filename stem.
/// - `words!(ADJECTIVES => "path/to/whatever.txt")` — explicit name.
struct WordsInput {
    name: Option<Ident>,
    path: LitStr,
}

impl Parse for WordsInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(Ident) && input.peek2(Token![=>]) {
            let name: Ident = input.parse()?;
            input.parse::<Token![=>]>()?;
            let path: LitStr = input.parse()?;
            Ok(WordsInput { name: Some(name), path })
        } else {
            let path: LitStr = input.parse()?;
            Ok(WordsInput { name: None, path })
        }
    }
}

/// Generate a static word list from a text file.
///
/// The macro reads the file at compile time, splits on whitespace,
/// deduplicates, and sorts the words. It produces a `pub static` array.
///
/// ```ignore
/// // Name derived from filename stem:
/// petname::words!("words/small/adjectives.txt");
/// // expands to:
/// // const _: &str = include_str!(...);
/// // pub static ADJECTIVES: [&str; 449] = ["able", "above", ...];
///
/// // Explicit name:
/// petname::words!(adjectives => "words/small/adjectives.txt");
/// // same expansion as above.
/// ```
#[proc_macro]
pub fn words(input: TokenStream) -> TokenStream {
    let parsed: WordsInput = syn::parse(input).expect("words! expects `\"path\"` or `NAME => \"path\"`");
    let relative_path = parsed.path.value();

    // Resolve the file relative to the calling crate's manifest dir.
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let file_path = PathBuf::from(&manifest_dir).join(&relative_path);

    // Read and process words.
    let contents = std::fs::read_to_string(&file_path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {e}", file_path.display()));
    let words = split_words_deduplicate_and_sort(&contents);

    // Determine the static name.
    let static_ident = match parsed.name {
        Some(ident) => ident,
        None => {
            let stem = file_path.file_stem().expect("file has no stem").to_string_lossy().to_uppercase();
            format_ident!("{}", stem)
        }
    };
    let word_count = words.len();

    // Emit file-tracking const + the static array.
    let expanded: TokenStream2 = quote! {
        const _: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/", #relative_path));
        pub static #static_ident: [&str; #word_count] = [ #( #words ),* ];
    };

    expanded.into()
}

fn split_words_deduplicate_and_sort(input: &str) -> Vec<String> {
    let words = input.split_whitespace().collect::<HashSet<_>>();
    let mut words: Vec<String> = words.into_iter().map(|s| s.to_owned()).collect();
    words.sort();
    words
}
