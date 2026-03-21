# Notes for AI agents

## Project overview

`rust-petname` is a Rust crate (library + CLI binary) for generating human-readable random names like _remarkably-striking-cricket_. It is a reimplementation of Dustin Kirkland's [petname](https://github.com/dustinkirkland/petname) with additional features.

The crate is `no_std` (with `extern crate alloc`). Feature flags:
- `default-rng` – enables `rand`'s default thread-local RNG
- `default-words` – embeds the built-in word lists via the `petnames!` macro
- `macros` – enables the `petnames!` proc macro (in the `petname-macros` subcrate)

## Key types

- `Petnames<'a>` – holds three word lists (adjectives, adverbs, nouns) as `Cow<'a, [&'a str]>`. Implements `Generator`.
- `Alliterations<'a>` – a `BTreeMap<char, Petnames<'a>>` grouping words by first letter for alliterative generation. Implements `Generator`.
- `Generator` – trait with a single required method `generate_into`. Object-safe: takes `&mut dyn rand::Rng`.
- `Namer<'a, G>` – a config struct (word count, separator, reference to a `Generator`). Created by `Petnames::namer` or `Alliterations::namer`. Has `generate_into` and `iter` methods. No trait import needed to use it.

`Namer::generate_into` writes into a caller-supplied `String` buffer (more efficient). `Namer::iter` yields owned `String`s via `core::iter::from_fn`.

## Testing

Run tests with `cargo hack --feature-powerset test` to cover all feature combinations. CI uses `cargo hack` for this purpose.

## Writing style

When writing or editing Markdown intended for **GitHub** (PR descriptions, issue bodies, README prose):
- Write paragraphs **unwrapped** – no hard line breaks within a paragraph. GitHub renders newlines as `<br>`, so wrapping text at 80 chars produces ragged output. Let the browser wrap.
- Use ` – ` (space + en-dash + space) rather than ` — ` (space + em-dash + space).

Terminal output and code comments may still be wrapped at ~100 chars.
