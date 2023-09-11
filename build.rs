use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("words.rs");

    let mut lines: Vec<String> = vec![];

    let list_sizes = ["small", "medium", "large"];
    let list_names = ["adjectives", "adverbs", "nouns"];

    for list_size in list_sizes {
        lines.push(format!("pub mod {list_size} {{"));
        for list_name in list_names {
            let list_path = format!("words/{list_size}/{list_name}.txt");
            println!("cargo:rerun-if-changed={list_path}");
            let list_raw = fs::read_to_string(list_path).unwrap();
            let list = list_raw.split_whitespace().collect::<Vec<_>>();
            lines.push(format!("  pub static {}: [&str; {}] = [", list_name.to_uppercase(), list.len()));
            lines.extend(list.iter().map(|word| format!("    \"{word}\",")));
            lines.push("  ];".to_string());
        }
        lines.push("}".to_string());
    }

    fs::write(dest_path, lines.join("\n")).unwrap();
    println!("cargo:rerun-if-changed=build.rs");
}
