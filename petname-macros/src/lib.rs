use proc_macro::TokenStream;
use std::borrow::Cow;
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
                        format!(
                            "unexpected argument `{other}`, expected `adjectives`, `adverbs`, or `nouns`"
                        ),
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

/// Paths resolved from input to the `petnames!` macro.
///
/// These can be relative paths, because they have not yet been anchored to
/// `CARGO_MANIFEST_DIR` for example, or absolute, like after a call to
/// [`Self::resolve`].
struct PetnamesPaths {
    adjectives: PathBuf,
    adverbs: PathBuf,
    nouns: PathBuf,
}

impl From<PetnamesInput> for PetnamesPaths {
    fn from(input: PetnamesInput) -> Self {
        fn value_or<'a>(value: Option<&'_ LitStr>, default: &'a str) -> Cow<'a, str> {
            value.map(LitStr::value).map(Cow::from).unwrap_or_else(|| default.into())
        }

        let path_adjectives = value_or(input.adjectives.as_ref(), "adjectives.txt");
        let path_adverbs = value_or(input.adverbs.as_ref(), "adverbs.txt");
        let path_nouns = value_or(input.nouns.as_ref(), "nouns.txt");

        match input.dir.as_ref().map(LitStr::value).map(PathBuf::from) {
            Some(base) => PetnamesPaths {
                adjectives: base.join(path_adjectives.as_ref()),
                adverbs: base.join(path_adverbs.as_ref()),
                nouns: base.join(path_nouns.as_ref()),
            },
            None => PetnamesPaths {
                adjectives: path_adjectives.as_ref().into(),
                adverbs: path_adverbs.as_ref().into(),
                nouns: path_nouns.as_ref().into(),
            },
        }
    }
}

impl PetnamesPaths {
    fn resolve(mut self, path: &Path) -> Self {
        self.adjectives = path.join(self.adjectives);
        self.adverbs = path.join(self.adverbs);
        self.nouns = path.join(self.nouns);
        self
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
///     adverbs = "adverbs.txt",
///     nouns = "nouns.txt",
/// );
/// ```
#[proc_macro]
pub fn petnames(input: TokenStream) -> TokenStream {
    let input: PetnamesInput = syn::parse(input).expect("petnames! parse error");

    let manifest_dir = std::env::var_os("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let manifest_path = PathBuf::from(&manifest_dir);
    let paths = PetnamesPaths::from(input).resolve(&manifest_path);

    let (adj_words, adj_count) = read_and_process(&paths.adjectives);
    let (adv_words, adv_count) = read_and_process(&paths.adverbs);
    let (noun_words, noun_count) = read_and_process(&paths.nouns);

    fn path_str<'a>(path: &'a Path, name: &'static str) -> &'a str {
        path.to_str().unwrap_or_else(|| panic!("{name} path not UTF-8: {}", path.display()))
    }

    let adj_path = path_str(&paths.adjectives, "adjectives");
    let adv_path = path_str(&paths.adverbs, "adverbs");
    let noun_path = path_str(&paths.nouns, "nouns");

    let expanded: TokenStream2 = quote! {
        {
            // This macro reads word list files via `std::fs`, but the compiler
            // doesn't track those reads. These `include_str!(…)` calls register
            // the files as dependencies so that changes trigger a rebuild. The
            // results are discarded. While `proc_macro::tracked_path::path` is
            // unstable, this is the idiomatic workaround.
            const _: &'static str = include_str!(#adj_path);
            const _: &'static str = include_str!(#adv_path);
            const _: &'static str = include_str!(#noun_path);
            // This is where the word lists are actually embedded.
            static ADJECTIVES: [&'static str; #adj_count] = [ #( #adj_words ),* ];
            static ADVERBS: [&'static str; #adv_count] = [ #( #adv_words ),* ];
            static NOUNS: [&'static str; #noun_count] = [ #( #noun_words ),* ];
            ::petname::Petnames {
                adjectives: ::alloc::borrow::Cow::Borrowed(&ADJECTIVES[..]),
                adverbs: ::alloc::borrow::Cow::Borrowed(&ADVERBS[..]),
                nouns: ::alloc::borrow::Cow::Borrowed(&NOUNS[..]),
            }
        }
    };

    expanded.into()
}

fn read_and_process(path: &Path) -> (Vec<String>, usize) {
    let contents =
        std::fs::read_to_string(path).unwrap_or_else(|e| panic!("could not read {}: {e}", path.display()));
    let words = split_words_deduplicate_and_sort(&contents);
    let count = words.len();
    (words, count)
}

fn split_words_deduplicate_and_sort(input: &str) -> Vec<String> {
    let words = input.split_whitespace().collect::<HashSet<_>>();
    let mut words: Vec<String> = words.into_iter().map(|s| s.to_owned()).collect();
    words.sort();
    words
}
