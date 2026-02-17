use proc_macro::TokenStream;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{Ident, LitStr, Token};

/// Input to the `petnames!` macro.
///
/// Supports two forms:
/// - `petnames!(dir = "words/small")` — finds adjectives.txt, adverbs.txt,
///   nouns.txt in the given directory.
/// - `petnames!(adjectives = "...", adverbs = "...", nouns = "...")` — explicit
///   paths for each word list.
enum PetnamesInput {
    Dir(LitStr),
    Explicit { adjectives: LitStr, adverbs: LitStr, nouns: LitStr },
}

impl Parse for PetnamesInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let first: Ident = input.parse()?;
        input.parse::<Token![=]>()?;
        let first_value: LitStr = input.parse()?;

        if first == "dir" {
            Ok(PetnamesInput::Dir(first_value))
        } else {
            // Parse remaining named arguments in any order.
            let mut adjectives = None;
            let mut adverbs = None;
            let mut nouns = None;

            // Assign the first argument.
            match first.to_string().as_str() {
                "adjectives" => adjectives = Some(first_value),
                "adverbs" => adverbs = Some(first_value),
                "nouns" => nouns = Some(first_value),
                other => {
                    let message = format!(
                        "unexpected argument `{other}`, expected `dir`, `adjectives`, `adverbs`, or `nouns`"
                    );
                    return Err(syn::Error::new(first.span(), message));
                }
            }

            // Parse the rest.
            while input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
                if input.is_empty() {
                    break; // trailing comma
                }
                let name: Ident = input.parse()?;
                input.parse::<Token![=]>()?;
                let value: LitStr = input.parse()?;
                match name.to_string().as_str() {
                    "adjectives" if adjectives.is_none() => adjectives = Some(value),
                    "adverbs" if adverbs.is_none() => adverbs = Some(value),
                    "nouns" if nouns.is_none() => nouns = Some(value),
                    "adjectives" | "adverbs" | "nouns" => {
                        return Err(syn::Error::new(name.span(), format!("duplicate argument `{name}`")));
                    }
                    other => {
                        let message = format!(
                            "unexpected argument `{other}`, expected `adjectives`, `adverbs`, or `nouns`"
                        );
                        return Err(syn::Error::new(name.span(), message));
                    }
                }
            }

            Ok(PetnamesInput::Explicit {
                adjectives: adjectives
                    .ok_or_else(|| syn::Error::new(input.span(), "missing `adjectives` argument"))?,
                adverbs: adverbs
                    .ok_or_else(|| syn::Error::new(input.span(), "missing `adverbs` argument"))?,
                nouns: nouns.ok_or_else(|| syn::Error::new(input.span(), "missing `nouns` argument"))?,
            })
        }
    }
}

/// Construct a [`Petnames`] from word list files at compile time.
///
/// ```ignore
/// // Directory form — looks for adjectives.txt, adverbs.txt, nouns.txt:
/// let p = petname::petnames!(dir = "words/small");
///
/// // Explicit form — specify each file path:
/// let p = petname::petnames!(
///     adjectives = "words/small/adjectives.txt",
///     adverbs = "words/small/adverbs.txt",
///     nouns = "words/small/nouns.txt",
/// );
/// ```
#[proc_macro]
pub fn petnames(input: TokenStream) -> TokenStream {
    let parsed: PetnamesInput = syn::parse(input).expect("petnames! parse error");
    let manifest_dir = std::env::var_os("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let base = PathBuf::from(&manifest_dir);

    let (adj_rel, adv_rel, noun_rel) = match &parsed {
        PetnamesInput::Dir(dir) => {
            let d = dir.value();
            (format!("{d}/adjectives.txt"), format!("{d}/adverbs.txt"), format!("{d}/nouns.txt"))
        }
        PetnamesInput::Explicit { adjectives, adverbs, nouns } => {
            (adjectives.value(), adverbs.value(), nouns.value())
        }
    };

    let adj_words = read_and_process(&base, &adj_rel);
    let adv_words = read_and_process(&base, &adv_rel);
    let noun_words = read_and_process(&base, &noun_rel);

    let adj_count = adj_words.len();
    let adv_count = adv_words.len();
    let noun_count = noun_words.len();

    let expanded: TokenStream2 = quote! {
        {
            // This macro reads word list files via `std::fs`, but the compiler
            // doesn't track those reads. These `include_str!(…)` calls register
            // the files as dependencies so that changes trigger a rebuild. The
            // results are discarded. While `proc_macro::tracked_path::path` is
            // unstable, this is the idiomatic workaround.
            const _: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/", #adj_rel));
            const _: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/", #adv_rel));
            const _: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/", #noun_rel));
            static ADJECTIVES: [&str; #adj_count] = [ #( #adj_words ),* ];
            static ADVERBS: [&str; #adv_count] = [ #( #adv_words ),* ];
            static NOUNS: [&str; #noun_count] = [ #( #noun_words ),* ];
            ::petname::Petnames {
                adjectives: ::alloc::borrow::Cow::Borrowed(&ADJECTIVES[..]),
                adverbs: ::alloc::borrow::Cow::Borrowed(&ADVERBS[..]),
                nouns: ::alloc::borrow::Cow::Borrowed(&NOUNS[..]),
            }
        }
    };

    expanded.into()
}

fn read_and_process(base: &Path, relative_path: &str) -> Vec<String> {
    let file_path = base.join(relative_path);
    let contents = std::fs::read_to_string(&file_path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {e}", file_path.display()));
    split_words_deduplicate_and_sort(&contents)
}

fn split_words_deduplicate_and_sort(input: &str) -> Vec<String> {
    let words = input.split_whitespace().collect::<HashSet<_>>();
    let mut words: Vec<String> = words.into_iter().map(|s| s.to_owned()).collect();
    words.sort();
    words
}
