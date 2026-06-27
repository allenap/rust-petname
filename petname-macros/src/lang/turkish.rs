use proc_macro::TokenStream;
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

use crate::{
    input::PetnamesInput,
    paths::PetnamesPaths,
    text::{read_and_process, word_tokens},
};

/// See [`turkish!`][`crate::turkish!`] for documentation.
pub fn expand(input: TokenStream) -> TokenStream {
    let input: PetnamesInput = syn::parse(input).expect("turkish! parse error");

    let manifest_dir = std::env::var_os("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let manifest_path = PathBuf::from(&manifest_dir);
    let paths = PetnamesPaths::from(input).resolve(&manifest_path);

    let (adjectives, adj_count) = read_and_process_adjectives(&paths.adjectives);
    let (adv_words, adv_count) = read_and_process(&paths.adverbs);
    let (noun_words, noun_count) = read_and_process(&paths.nouns);

    fn path_str<'a>(path: &'a Path, name: &'static str) -> &'a str {
        path.to_str().unwrap_or_else(|| panic!("{name} path not UTF-8: {}", path.display()))
    }

    let adj_path = path_str(&paths.adjectives, "adjectives");
    let adv_path = path_str(&paths.adverbs, "adverbs");
    let noun_path = path_str(&paths.nouns, "nouns");

    let adj_items: Vec<TokenStream2> = adjectives
        .iter()
        .map(|(word, emphatic)| {
            let emphatic = match emphatic {
                Some(emphatic) => quote! { ::core::option::Option::Some(#emphatic) },
                None => quote! { ::core::option::Option::None },
            };
            quote! { ::petname::lang::turkish::Adjective { word: #word, emphatic: #emphatic } }
        })
        .collect();

    let expanded: TokenStream2 = quote! {
        {
            // See the note in `petnames!` about `include_str!` being used purely
            // to register these files as rebuild dependencies.
            const _: &'static str = include_str!(#adj_path);
            const _: &'static str = include_str!(#adv_path);
            const _: &'static str = include_str!(#noun_path);
            // This is where the word lists are actually embedded.
            static ADJECTIVES: [::petname::lang::turkish::Adjective<'static>; #adj_count] = [ #( #adj_items ),* ];
            static ADVERBS: [&'static str; #adv_count] = [ #( #adv_words ),* ];
            static NOUNS: [&'static str; #noun_count] = [ #( #noun_words ),* ];
            ::petname::lang::turkish::Petnames {
                adjectives: ::alloc::borrow::Cow::Borrowed(&ADJECTIVES[..]),
                adverbs: ::alloc::borrow::Cow::Borrowed(&ADVERBS[..]),
                nouns: ::alloc::borrow::Cow::Borrowed(&NOUNS[..]),
            }
        }
    };

    expanded.into()
}

/// Read adjectives, each an optional `base=emphatic` token, deduplicated by
/// base form and sorted.
fn read_and_process_adjectives(path: &Path) -> (Vec<(String, Option<String>)>, usize) {
    let contents =
        std::fs::read_to_string(path).unwrap_or_else(|e| panic!("could not read {}: {e}", path.display()));
    let mut seen = HashSet::new();
    let mut adjectives: Vec<(String, Option<String>)> = Vec::new();
    for token in word_tokens(&contents) {
        let (base, emphatic) = match token.split_once('=') {
            Some((base, emphatic)) => (base.to_owned(), Some(emphatic.to_owned())),
            None => (token.to_owned(), None),
        };
        if seen.insert(base.clone()) {
            adjectives.push((base, emphatic));
        }
    }
    adjectives.sort();
    let count = adjectives.len();
    (adjectives, count)
}
