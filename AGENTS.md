# Notes for AI agents

## Project overview

`rust-petname` is a Rust crate (library + CLI binary) for generating human-readable random names like _remarkably-striking-cricket_. It is a reimplementation of Dustin Kirkland's [petname](https://github.com/dustinkirkland/petname) with additional features.

The crate is `no_std` (with `extern crate alloc`). Feature flags:
- `default-rng` – enables `rand`'s default thread-local RNG
- `default-words` – embeds the built-in word lists via the `english!` macro
- `macros` – enables the `english!` proc macro and its `petnames!` alias (in the `petname-macros` subcrate)
- `lang-turkish` (not a default) – compiles the `lang::turkish` module and enables `--language turkish`. The built-in lists (`lang::turkish::Petnames::small`, via the `turkish!` macro) are embedded only when `default-words` is also on, mirroring the English lists

## Key types

- `Petnames<'a>` – lives in `lang::english`, re-exported at the crate root as `petname::Petnames`. Holds three word lists (adjectives, adverbs, nouns) as `Words<'a>` (= `Cow<'a, [&'a str]>`). Implements `Generator`.
- `Alliterations<'a>` – a `BTreeMap<char, Petnames<'a>>` grouping words by first letter for alliterative generation. Implements `Generator`.
- `Generator` – trait with a single required method `generate_into`. Object-safe: takes `&mut dyn rand::Rng`.
- `Namer<'a, G>` – a config struct (word count, separator, reference to a `Generator`). Created by `Petnames::namer` or `Alliterations::namer`. Has `generate_into` and `iter` methods. No trait import needed to use it.

## Languages

Non-English languages live in `src/lang/` – each is a **distinct type** implementing `Generator` (we deliberately duplicate rather than abstract until shared structure is proven). `lang::turkish::Petnames<'a>` is the first: like the English `lang::english::Petnames` but its adjectives are `Adjective { word, emphatic: Option<&str> }` to carry _pekiştirme_ reduplication (e.g. `kırmızı`→`kıpkırmızı`), used only in two-word names. Word lists are under `words/turkish/` and embedded by the `turkish!` macro; the adjectives file uses `base=emphatic` tokens, and `#` begins a line comment (the `english!`/`turkish!` tokenizer now strips `#` comments for all word files). Roadmap: Luxembourgish, French, German, then Spanish/Italian/Portuguese – the later ones add gender agreement, word-order, and join-time sandhi (see `Generator` owning the buffer).

`Namer::generate_into` writes into a caller-supplied `String` buffer (more efficient). `Namer::iter` yields owned `String`s via `core::iter::from_fn`.

## Testing

Run tests with `cargo hack --feature-powerset test` to cover all feature combinations. CI uses `cargo hack` for this purpose.

## Docs

Run `scripts/doc` (optionally with `--open`) to build the API docs the way docs.rs does: nightly rustdoc, `--all-features`, and `--cfg docsrs` so that `doc_cfg` labels each gated item with the feature that enables it. It needs the nightly toolchain (`rustup toolchain install nightly`). This is also the build under which the intra-doc links to feature-gated items (e.g. `lang::turkish`) resolve – a plain `cargo doc` (default features) will warn about those, which is expected.

## Writing style

When writing or editing Markdown intended for **GitHub** (PR descriptions, issue bodies, README prose):
- Write paragraphs **unwrapped** – no hard line breaks within a paragraph. GitHub renders newlines as `<br>`, so wrapping text at 80 chars produces ragged output. Let the browser wrap.
- Use ` – ` (space + en-dash + space) rather than ` — ` (space + em-dash + space).

Terminal output and code comments may still be wrapped at ~100 chars.
