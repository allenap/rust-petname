use proc_macro::TokenStream;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{Ident, LitStr, Token};

/// Input to the `petnames!` macro.
///
/// An optional unnamed directory prefix may be followed by named arguments
/// for each word list. Individual paths are resolved relative to the directory
/// if one is given, or used directly otherwise.
struct PetnamesInput {
    dir: Option<LitStr>,
    adjectives: Option<LitStr>,
    adverbs: Option<LitStr>,
    nouns: Option<LitStr>,
}

impl Parse for PetnamesInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut dir = None;
        let mut adjectives = None;
        let mut adverbs = None;
        let mut nouns = None;

        // Optional unnamed directory argument.
        if input.peek(LitStr) {
            dir = Some(input.parse::<LitStr>()?);
            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        // Named arguments in any order.
        while !input.is_empty() {
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
                    return Err(syn::Error::new(
                        name.span(),
                        format!("unexpected argument `{other}`, expected `adjectives`, `adverbs`, or `nouns`"),
                    ));
                }
            }
            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(PetnamesInput { dir, adjectives, adverbs, nouns })
    }
}

impl PetnamesInput {
    /// Resolve a relative path for a word list, using the directory prefix if given.
    ///
    /// Returns an error if neither a directory nor an explicit path is provided.
    fn resolve(&self, name: &str, path: &Option<LitStr>, default_filename: &str) -> Result<String, String> {
        match (&self.dir, path) {
            (Some(d), Some(p)) => Ok(format!("{}/{}", d.value(), p.value())),
            (Some(d), None) => Ok(format!("{}/{}", d.value(), default_filename)),
            (None, Some(p)) => Ok(p.value()),
            (None, None) => Err(format!("missing `{name}` argument (or an unnamed directory)")),
        }
    }
}

/// Construct a [`Petnames`] from word list files at compile time.
///
/// ```ignore
/// // Unnamed directory — looks for adjectives.txt, adverbs.txt, nouns.txt:
/// let p = petname::petnames!("words/small");
///
/// // Named arguments — specify each file path explicitly:
/// let p = petname::petnames!(
///     adjectives = "words/small/adjectives.txt",
///     adverbs = "words/small/adverbs.txt",
///     nouns = "words/small/nouns.txt",
/// );
///
/// // Directory prefix with overrides — paths are resolved relative to the directory:
/// let p = petname::petnames!(
///     "words/small",
///     adjectives = "adjectives.txt",
///     nouns = "nouns.txt",
///     adverbs = "adverbs.txt",
/// );
/// ```
#[proc_macro]
pub fn petnames(input: TokenStream) -> TokenStream {
    let parsed: PetnamesInput = syn::parse(input).expect("petnames! parse error");

    let adj_rel = parsed.resolve("adjectives", &parsed.adjectives, "adjectives.txt")
        .expect("petnames! argument error");
    let adv_rel = parsed.resolve("adverbs", &parsed.adverbs, "adverbs.txt")
        .expect("petnames! argument error");
    let noun_rel = parsed.resolve("nouns", &parsed.nouns, "nouns.txt")
        .expect("petnames! argument error");

    let manifest_dir = std::env::var_os("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let base = PathBuf::from(&manifest_dir);

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
