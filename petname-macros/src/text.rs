use std::collections::HashSet;
use std::path::Path;

pub fn read_and_process(path: &Path) -> (Vec<String>, usize) {
    let contents =
        std::fs::read_to_string(path).unwrap_or_else(|e| panic!("could not read {}: {e}", path.display()));
    let words = split_words_deduplicate_and_sort(&contents);
    let count = words.len();
    (words, count)
}

fn split_words_deduplicate_and_sort(input: &str) -> Vec<String> {
    let words = word_tokens(input).collect::<HashSet<_>>();
    let mut words: Vec<String> = words.into_iter().map(|s| s.to_owned()).collect();
    words.sort();
    words
}

/// Yield whitespace-delimited word tokens, ignoring `#` line comments. A `#`
/// begins a comment that runs to the end of the line. No built-in word contains
/// `#`, so this is safe for the existing word lists.
pub fn word_tokens(input: &str) -> impl Iterator<Item = &str> {
    input.lines().flat_map(|line| {
        let line = line.split_once('#').map_or(line, |(before, _)| before);
        line.split_whitespace()
    })
}
