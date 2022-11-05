use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("words.rs");

    let list_paths = [
        "words/small/adjectives.txt",
        "words/small/adverbs.txt",
        "words/small/names.txt",
        "words/medium/adjectives.txt",
        "words/medium/adverbs.txt",
        "words/medium/names.txt",
        "words/large/adjectives.txt",
        "words/large/adverbs.txt",
        "words/large/names.txt",
    ];

    let mut lines: Vec<String> = vec![];

    for list_path in list_paths {
        println!("cargo:rerun-if-changed={list_path}");
        let list_raw = fs::read_to_string(list_path).unwrap();
        let list = list_raw.split_whitespace().collect::<Vec<_>>();
        let list_path_parts = list_path.split(&['/', '.']).collect::<Vec<_>>();
        let [_, list_size, list_name, _] = list_path_parts[..] else { panic!() };
        lines.push(format!("const {list_name}_{list_size}: [&str; _] = ["));
        lines.extend(list.iter().map(|word| format!("    \"{word}\",")));
        lines.push("];".into());
    }

    fs::write(&dest_path, lines.join("\n")).unwrap();
    println!("cargo:rerun-if-changed=build.rs");
}
