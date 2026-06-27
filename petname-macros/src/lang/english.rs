use proc_macro::TokenStream;
use std::path::{Path, PathBuf};

use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

use crate::{input::PetnamesInput, paths::PetnamesPaths, text::read_and_process};

/// See [`english!`][`crate::english!`] for documentation.
pub fn expand(input: TokenStream) -> TokenStream {
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
            ::petname::lang::english::Petnames {
                adjectives: ::alloc::borrow::Cow::Borrowed(&ADJECTIVES[..]),
                adverbs: ::alloc::borrow::Cow::Borrowed(&ADVERBS[..]),
                nouns: ::alloc::borrow::Cow::Borrowed(&NOUNS[..]),
            }
        }
    };

    expanded.into()
}
