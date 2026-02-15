use proc_macro::TokenStream;
use std::collections::HashSet;
use std::path::PathBuf;

use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};

/// Generate a static word list from a text file.
///
/// The macro reads the file at compile time, splits on whitespace,
/// deduplicates, and sorts the words. It produces a `pub static` array
/// named after the file stem in uppercase.
///
/// ```ignore
/// petname_macros::word_list!("words/small/adjectives.txt");
/// // expands to:
/// // const _: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/", "words/small/adjectives.txt"));
/// // pub static ADJECTIVES: [&str; 449] = ["able", "above", ...];
/// ```
#[proc_macro]
pub fn word_list(input: TokenStream) -> TokenStream {
    let lit: syn::LitStr = syn::parse(input).expect("word_list! expects a string literal argument");
    let relative_path = lit.value();

    // Resolve the file relative to the calling crate's manifest dir.
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR not set");
    let file_path = PathBuf::from(&manifest_dir).join(&relative_path);

    // Read and process words.
    let contents = std::fs::read_to_string(&file_path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {e}", file_path.display()));
    let words = split_words_deduplicate_and_sort(&contents);

    // Derive the static name from the filename stem, uppercased.
    let stem = file_path
        .file_stem()
        .expect("file has no stem")
        .to_string_lossy()
        .to_uppercase();
    let static_ident = format_ident!("{}", stem);
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
