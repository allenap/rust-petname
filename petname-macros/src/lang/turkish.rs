use proc_macro::TokenStream;
use std::{collections::HashSet, path::Path};

use quote::quote;

use crate::{
    lang::{embed, resolve, WordList},
    text::{read_and_process, word_tokens},
};

/// See [`turkish!`][`crate::turkish!`] for documentation.
pub fn expand(input: TokenStream, macro_name: &str) -> TokenStream {
    let paths = resolve(input, macro_name);
    embed(
        "turkish",
        &paths,
        read_and_process_adjectives(&paths.adjectives),
        WordList::strings(read_and_process(&paths.adverbs)),
        WordList::strings(read_and_process(&paths.nouns)),
    )
}

/// Read adjectives, each an optional `base=emphatic` token, deduplicated by
/// base form and sorted.
fn read_and_process_adjectives(path: &Path) -> WordList {
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

    WordList::values(
        quote! { ::petname::lang::turkish::Adjective<'static> },
        adjectives
            .iter()
            .map(|(word, emphatic)| {
                let emphatic = match emphatic {
                    Some(emphatic) => quote! { ::core::option::Option::Some(#emphatic) },
                    None => quote! { ::core::option::Option::None },
                };
                quote! { ::petname::lang::turkish::Adjective { word: #word, emphatic: #emphatic } }
            })
            .collect(),
    )
}
