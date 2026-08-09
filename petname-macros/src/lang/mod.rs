//! Language-specific petname macros.
//!
//! Each language module owns the part that genuinely differs between languages
//! – how its word list files tokenize – and shares the scaffolding here:
//! resolving the paths given to a macro, and emitting the statics that the
//! generated `Petnames` borrows from.

use proc_macro::TokenStream;
use std::path::Path;

use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};

use crate::{input::PetnamesInput, paths::PetnamesPaths};

pub(crate) mod english;
pub(crate) mod luxembourgish;
pub(crate) mod turkish;

/// One word list, ready to embed: the element type of the `static` array it
/// becomes, and the rendered items that fill it.
pub(crate) struct WordList {
    element: TokenStream2,
    items: Vec<TokenStream2>,
}

impl WordList {
    /// A list of plain words, embedded as `&'static str`.
    pub(crate) fn strings(words: Vec<String>) -> Self {
        Self { element: quote! { &'static str }, items: words.iter().map(|word| quote!(#word)).collect() }
    }

    /// A list of `element`-typed values, for a language that annotates its
    /// words – with a gender, say, or an alternative form.
    pub(crate) fn values(element: TokenStream2, items: Vec<TokenStream2>) -> Self {
        Self { element, items }
    }
}

/// Resolve the paths given to a macro, anchoring them at the build-time
/// `CARGO_MANIFEST_DIR`, i.e. the directory of the `Cargo.toml` of the crate
/// being compiled. `macro_name` appears in the error if the input won't parse.
pub(crate) fn resolve(input: TokenStream, macro_name: &str) -> PetnamesPaths {
    let input: PetnamesInput =
        syn::parse(input).unwrap_or_else(|error| panic!("{macro_name}! parse error: {error}"));
    let manifest_dir = std::env::var_os("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    PetnamesPaths::from(input).resolve(Path::new(&manifest_dir))
}

/// Emit the expression that embeds three word lists and constructs the
/// `Petnames` of `lang`, e.g. `::petname::lang::english::Petnames`.
pub(crate) fn embed(
    lang: &str,
    paths: &PetnamesPaths,
    adjectives: WordList,
    adverbs: WordList,
    nouns: WordList,
) -> TokenStream {
    let lang = format_ident!("{lang}");

    let adj_path = path_str(&paths.adjectives, "adjectives");
    let adv_path = path_str(&paths.adverbs, "adverbs");
    let noun_path = path_str(&paths.nouns, "nouns");

    let WordList { element: adj_element, items: adj_items } = adjectives;
    let WordList { element: adv_element, items: adv_items } = adverbs;
    let WordList { element: noun_element, items: noun_items } = nouns;
    let (adj_count, adv_count, noun_count) = (adj_items.len(), adv_items.len(), noun_items.len());

    let expanded: TokenStream2 = quote! {
        {
            // These macros read word list files via `std::fs`, but the compiler
            // doesn't track those reads. These `include_str!(…)` calls register
            // the files as dependencies so that changes trigger a rebuild. The
            // results are discarded. While `proc_macro::tracked_path::path` is
            // unstable, this is the idiomatic workaround.
            const _: &'static str = include_str!(#adj_path);
            const _: &'static str = include_str!(#adv_path);
            const _: &'static str = include_str!(#noun_path);
            // This is where the word lists are actually embedded.
            static ADJECTIVES: [#adj_element; #adj_count] = [ #( #adj_items ),* ];
            static ADVERBS: [#adv_element; #adv_count] = [ #( #adv_items ),* ];
            static NOUNS: [#noun_element; #noun_count] = [ #( #noun_items ),* ];
            ::petname::lang::#lang::Petnames {
                adjectives: ::alloc::borrow::Cow::Borrowed(&ADJECTIVES[..]),
                adverbs: ::alloc::borrow::Cow::Borrowed(&ADVERBS[..]),
                nouns: ::alloc::borrow::Cow::Borrowed(&NOUNS[..]),
            }
        }
    };

    expanded.into()
}

fn path_str<'a>(path: &'a Path, name: &'static str) -> &'a str {
    path.to_str().unwrap_or_else(|| panic!("{name} path not UTF-8: {}", path.display()))
}
