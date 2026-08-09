//! The word-list macros' input forms.
//!
//! The built-in lists all use the single-directory form, so without these the
//! named-argument half of the macros' documented interface – and the input
//! parser that backs it – is never exercised.

#![cfg(all(feature = "macros", feature = "default-words"))]

// The macros expand to `::alloc::borrow::Cow::Borrowed(…)`, so every crate that
// uses them must have `alloc` in its extern prelude. `petname` itself does, so
// the built-in lists compile; nothing outside it does by default.
extern crate alloc;

use petname::Petnames;

#[test]
fn dir_only() {
    let petnames = petname::petnames!("words/small");
    assert_eq!(petnames, Petnames::small());
}

#[test]
fn named_paths_only() {
    let petnames = petname::petnames!(
        adjectives = "words/small/adjectives.txt",
        adverbs = "words/medium/adverbs.txt",
        nouns = "words/large/nouns.txt",
    );
    assert_eq!(petnames.adjectives, Petnames::small().adjectives);
    assert_eq!(petnames.adverbs, Petnames::medium().adverbs);
    assert_eq!(petnames.nouns, Petnames::large().nouns);
}

#[test]
fn dir_with_named_paths() {
    let petnames = petname::petnames!(
        "words",
        adjectives = "small/adjectives.txt",
        adverbs = "medium/adverbs.txt",
        nouns = "large/nouns.txt",
    );
    assert_eq!(petnames.adjectives, Petnames::small().adjectives);
    assert_eq!(petnames.adverbs, Petnames::medium().adverbs);
    assert_eq!(petnames.nouns, Petnames::large().nouns);
}

#[test]
fn dir_with_some_named_paths() {
    // Unnamed lists keep their default file names within the directory.
    let petnames = petname::petnames!("words/small", nouns = "../large/nouns.txt");
    assert_eq!(petnames.adjectives, Petnames::small().adjectives);
    assert_eq!(petnames.adverbs, Petnames::small().adverbs);
    assert_eq!(petnames.nouns, Petnames::large().nouns);
}

#[test]
fn named_paths_in_any_order() {
    let petnames = petname::petnames!(
        nouns = "words/small/nouns.txt",
        adjectives = "words/small/adjectives.txt",
        adverbs = "words/small/adverbs.txt",
    );
    assert_eq!(petnames, Petnames::small());
}

#[test]
fn trailing_comma_is_optional() {
    let petnames = petname::petnames!("words/small", nouns = "nouns.txt");
    assert_eq!(petnames, Petnames::small());
}

/// The other languages share the same input parser.
#[cfg(feature = "lang-turkish")]
#[test]
fn other_languages_take_the_same_forms() {
    let turkish = petname::turkish!("words/turkish", adjectives = "adjectives.txt");
    assert_eq!(turkish, petname::lang::turkish::Petnames::small());
}
