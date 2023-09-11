use anyhow::{Context, Result};
use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::Path;

fn main() -> Result<()> {
    let words_dir = Path::new("words");
    let out_dir = env::var_os("OUT_DIR").context("OUT_DIR not set")?;
    let dest_path = Path::new(&out_dir).join("words.rs");

    let mut lines: Vec<String> = vec![];

    let list_sizes = ["small", "medium", "large"];
    let list_names = ["adjectives", "adverbs", "nouns"];

    for list_size in list_sizes {
        lines.push(format!("pub mod {list_size} {{"));
        for list_name in list_names {
            let list_path = words_dir.join(list_size).join(list_name).with_extension("txt");
            println!("cargo:rerun-if-changed={}", list_path.to_string_lossy());
            let list_raw = fs::read_to_string(&list_path)
                .with_context(|| format!("Could not read word list from {list_path:?}"))?;
            let list = {
                // Ensure we have no duplicates.
                let words = list_raw.split_whitespace().collect::<HashSet<_>>();
                // Collect into a `Vec` and sort it.
                let mut list = words.into_iter().collect::<Vec<_>>();
                list.sort();
                list
            };
            lines.push(format!("  pub static {}: [&str; {}] = [", list_name.to_uppercase(), list.len()));
            lines.extend(list.iter().map(|word| format!("    \"{word}\",")));
            lines.push("  ];".to_string());
        }
        lines.push("}".to_string());
    }

    fs::write(&dest_path, lines.join("\n"))
        .with_context(|| format!("Could not write word lists to output file {dest_path:?}"))?;
    println!("cargo:rerun-if-changed=build.rs");

    Ok(())
}
