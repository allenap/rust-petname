use anyhow::{Context, Result};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::Path;

fn main() -> Result<()> {
    let words_dir = Path::new("words");
    let out_dir = env::var_os("OUT_DIR").context("OUT_DIR not set")?;
    let dest_path = Path::new(&out_dir).join("words.rs");

    let list_sizes = ["small", "medium", "large"];
    let list_names = ["adjectives", "adverbs", "nouns"];

    // Generate modules for each list size.
    let list_modules = list_sizes
        .into_iter()
        .map(|list_size| {
            // Generate static lists for each list name.
            let lists = list_names
                .into_iter()
                .map(|list_name| {
                    let words_path = words_dir.join(list_size).join(list_name).with_extension("txt");
                    // Remind Cargo to re-run if the word list changes.
                    println!("cargo:rerun-if-changed={}", words_path.to_string_lossy());
                    let words_raw = fs::read_to_string(&words_path)
                        .with_context(|| format!("Could not read word list from {words_path:?}"))?;
                    let words = split_words_deduplicate_and_sort(&words_raw);
                    let list_ident = format_ident!("{}", list_name.to_uppercase());
                    let list_len = words.len();
                    let list = quote! { pub static #list_ident: [&str; #list_len] = [ #( #words ),* ]; };
                    Ok(list)
                })
                .collect::<Result<Vec<TokenStream>>>()?;
            let module_ident = format_ident!("{}", list_size);
            let module_tokens = quote! { pub mod #module_ident { #(#lists)* } };
            Ok(module_tokens)
        })
        .collect::<Result<Vec<TokenStream>>>()?;

    // Assemble the complete `words` module.
    let modules = quote! { #(#list_modules)* };
    let module = modules.to_string();

    // Write the module to the output file.
    fs::write(&dest_path, module)
        .with_context(|| format!("Could not write word lists to output file {dest_path:?}"))?;

    // DEBUG: Print the generated file's path as a warning:
    // println!("cargo:warning={}", dest_path.display());

    // Remind Cargo when to re-run this build script.
    println!("cargo:rerun-if-changed=build.rs");

    Ok(())
}

fn split_words_deduplicate_and_sort(input: &str) -> Vec<&str> {
    // Ensure we have no duplicates.
    let words = input.split_whitespace().collect::<HashSet<_>>();
    // Collect into a `Vec` and sort it.
    let mut words = words.into_iter().collect::<Vec<_>>();
    words.sort();
    words
}
