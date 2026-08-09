use proc_macro::TokenStream;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

use crate::{
    input::PetnamesInput,
    paths::PetnamesPaths,
    text::{read_and_process, word_tokens},
};

/// See [`luxembourgish!`][`crate::luxembourgish!`] for documentation.
pub fn expand(input: TokenStream) -> TokenStream {
    let input: PetnamesInput = syn::parse(input).expect("luxembourgish! parse error");

    let manifest_dir = std::env::var_os("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let manifest_path = PathBuf::from(&manifest_dir);
    let paths = PetnamesPaths::from(input).resolve(&manifest_path);

    let (adjectives, adj_count) = read_and_process_adjectives(&paths.adjectives);
    let (adv_words, adv_count) = read_and_process(&paths.adverbs);
    let (nouns, noun_count) = read_and_process_nouns(&paths.nouns);

    fn path_str<'a>(path: &'a Path, name: &'static str) -> &'a str {
        path.to_str().unwrap_or_else(|| panic!("{name} path not UTF-8: {}", path.display()))
    }

    let adj_path = path_str(&paths.adjectives, "adjectives");
    let adv_path = path_str(&paths.adverbs, "adverbs");
    let noun_path = path_str(&paths.nouns, "nouns");

    let adj_items: Vec<TokenStream2> = adjectives
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
        .collect();

    let noun_items: Vec<TokenStream2> = nouns
        .iter()
        .map(|(word, gender)| {
            let gender = match gender {
                Gender::Masculine => quote! { ::petname::lang::luxembourgish::Gender::Masculine },
                Gender::Feminine => quote! { ::petname::lang::luxembourgish::Gender::Feminine },
                Gender::Neuter => quote! { ::petname::lang::luxembourgish::Gender::Neuter },
            };
            quote! { ::petname::lang::luxembourgish::Noun { word: #word, gender: #gender } }
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
            static ADJECTIVES: [::petname::lang::luxembourgish::Adjective<'static>; #adj_count] = [ #( #adj_items ),* ];
            static ADVERBS: [&'static str; #adv_count] = [ #( #adv_words ),* ];
            static NOUNS: [::petname::lang::luxembourgish::Noun<'static>; #noun_count] = [ #( #noun_items ),* ];
            ::petname::lang::luxembourgish::Petnames {
                adjectives: ::alloc::borrow::Cow::Borrowed(&ADJECTIVES[..]),
                adverbs: ::alloc::borrow::Cow::Borrowed(&ADVERBS[..]),
                nouns: ::alloc::borrow::Cow::Borrowed(&NOUNS[..]),
            }
        }
    };

    expanded.into()
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
fn read_and_process_adjectives(path: &Path) -> (Vec<(String, String, String)>, usize) {
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
    let count = adjectives.len();
    (adjectives, count)
}

/// Read nouns, each a `word:gender` token (gender one of `m`, `f`, `n`),
/// deduplicated by word and sorted. As with the adjectives, a repeat that
/// disagrees – here about the gender – is an error.
fn read_and_process_nouns(path: &Path) -> (Vec<(String, Gender)>, usize) {
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
    let count = nouns.len();
    (nouns, count)
}
