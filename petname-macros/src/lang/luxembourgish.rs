use proc_macro::TokenStream;
use std::{collections::HashMap, path::Path};

use quote::quote;

use crate::{
    lang::{embed, resolve, WordList},
    text::{read_and_process, word_tokens},
};

/// See [`luxembourgish!`][`crate::luxembourgish!`] for documentation.
pub fn expand(input: TokenStream, macro_name: &str) -> TokenStream {
    let paths = resolve(input, macro_name);
    embed(
        "luxembourgish",
        &paths,
        read_and_process_adjectives(&paths.adjectives),
        WordList::strings(read_and_process(&paths.adverbs)),
        read_and_process_nouns(&paths.nouns),
    )
}

/// Gender as parsed from the nouns file.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Gender {
    Masculine,
    Feminine,
    Neuter,
}

/// Read adjectives, each a `masculine:feminine:neuter` token, deduplicated by
/// the feminine (bare) form and sorted.
///
/// A repeated adjective is dropped, but a repeat that *disagrees* about the
/// other forms is an error rather than a silent coin-toss: these lists are meant
/// to be edited by hand, and a contradiction in them is a mistake worth naming.
fn read_and_process_adjectives(path: &Path) -> WordList {
    let contents =
        std::fs::read_to_string(path).unwrap_or_else(|e| panic!("could not read {}: {e}", path.display()));
    let mut seen: HashMap<String, (String, String)> = HashMap::new();
    let mut adjectives: Vec<(String, String, String)> = Vec::new();
    for token in word_tokens(&contents) {
        let mut parts = token.split(':');
        match (parts.next(), parts.next(), parts.next(), parts.next()) {
            (Some(masc), Some(fem), Some(neut), None)
                if !masc.is_empty() && !fem.is_empty() && !neut.is_empty() =>
            {
                match seen.get(fem) {
                    None => {
                        seen.insert(fem.to_owned(), (masc.to_owned(), neut.to_owned()));
                        adjectives.push((masc.to_owned(), fem.to_owned(), neut.to_owned()));
                    }
                    Some((seen_masc, seen_neut)) if seen_masc == masc && seen_neut == neut => {}
                    Some((seen_masc, seen_neut)) => panic!(
                        "adjective `{fem}` appears twice with different forms: \
                         `{seen_masc}:{fem}:{seen_neut}` and `{token}`"
                    ),
                }
            }
            _ => panic!(
                "adjective `{token}` must be `masculine:feminine:neuter` (three colon-separated, non-empty forms)"
            ),
        }
    }
    adjectives.sort_by(|a, b| a.1.cmp(&b.1));

    WordList::values(
        quote! { ::petname::lang::luxembourgish::Adjective<'static> },
        adjectives
            .iter()
            .map(|(masculine, feminine, neuter)| {
                quote! {
                    ::petname::lang::luxembourgish::Adjective {
                        masculine: #masculine,
                        feminine: #feminine,
                        neuter: #neuter,
                    }
                }
            })
            .collect(),
    )
}

/// Read nouns, each a `word:gender` token (gender one of `m`, `f`, `n`),
/// deduplicated by word and sorted. As with the adjectives, a repeat that
/// disagrees – here about the gender – is an error.
fn read_and_process_nouns(path: &Path) -> WordList {
    let contents =
        std::fs::read_to_string(path).unwrap_or_else(|e| panic!("could not read {}: {e}", path.display()));
    let mut seen: HashMap<String, Gender> = HashMap::new();
    let mut nouns: Vec<(String, Gender)> = Vec::new();
    for token in word_tokens(&contents) {
        let mut parts = token.split(':');
        let (word, gender) = match (parts.next(), parts.next(), parts.next()) {
            (Some(word), Some(gender), None) if !word.is_empty() => {
                let gender = match gender {
                    "m" => Gender::Masculine,
                    "f" => Gender::Feminine,
                    "n" => Gender::Neuter,
                    other => {
                        panic!("noun `{token}` has unknown gender `{other}` (expected one of m, f, n)")
                    }
                };
                (word.to_owned(), gender)
            }
            _ => panic!("noun `{token}` must be `word:gender` with gender one of m, f, n"),
        };
        match seen.get(&word) {
            None => {
                seen.insert(word.clone(), gender);
                nouns.push((word, gender));
            }
            Some(seen) if *seen == gender => {}
            Some(seen) => {
                panic!("noun `{word}` appears twice with different genders: {seen:?} and {gender:?}")
            }
        }
    }
    nouns.sort_by(|a, b| a.0.cmp(&b.0));

    WordList::values(
        quote! { ::petname::lang::luxembourgish::Noun<'static> },
        nouns
            .iter()
            .map(|(word, gender)| {
                let gender = match gender {
                    Gender::Masculine => quote! { ::petname::lang::luxembourgish::Gender::Masculine },
                    Gender::Feminine => quote! { ::petname::lang::luxembourgish::Gender::Feminine },
                    Gender::Neuter => quote! { ::petname::lang::luxembourgish::Gender::Neuter },
                };
                quote! { ::petname::lang::luxembourgish::Noun { word: #word, gender: #gender } }
            })
            .collect(),
    )
}
