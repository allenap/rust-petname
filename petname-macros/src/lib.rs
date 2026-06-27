use proc_macro::TokenStream;

mod input;
mod lang;
mod paths;
mod text;

/// Construct an English petname generator from word list files at compile time.
///
/// Word list files should be UTF-8, with words delimited by whitespace.
///
/// ⚠️ Note when reading the examples that all non-absolute paths are ultimately
/// resolved relative to the build-time `CARGO_MANIFEST_DIR`, i.e. the directory
/// of `Cargo.toml` for the crate being compiled.
///
/// # Examples
///
/// Given a directory path it will look for `adjectives.txt`, `adverbs.txt`, and
/// `nouns.txt` within that directory:
///
/// ```ignore
/// let p = petname::petnames!("words/small");
/// ```
///
/// One can specify each file path explicitly too:
///
/// ```ignore
/// let p = petname::petnames!(
///     adjectives = "words/small/adjectives.txt",
///     adverbs = "words/medium/adverbs.txt",
///     nouns = "words/large/nouns.txt",
/// );
/// ```
///
/// Given both a directory path and individual paths, the files' paths are
/// resolved relative to the directory:
///
/// ```ignore
/// let p = petname::petnames!(
///     "words",
///     adjectives = "small/adjectives.txt",
///     adverbs = "medium/adverbs.txt",
///     nouns = "large/nouns.txt",
/// );
/// ```
///
/// It is not necessary to specify all of the paths. This will look for
/// `adjectives.txt`, `adverbs.txt`, and `cars.txt` in the `words/other`
/// directory:
///
/// ```ignore
/// let p = petname::petnames!("words/other", nouns = "cars.txt");
/// ```
///
/// It is not necessary to specify _any_ of the paths. This will look for
/// `adjectives.txt`, `adverbs.txt`, and `nouns.txt` in the crate root
/// directory:
///
/// ```ignore
/// let p = petname::petnames!();
/// ```
///
#[proc_macro]
pub fn english(input: TokenStream) -> TokenStream {
    crate::lang::english::expand(input)
}

/// Construct a Turkish petname generator from word list files at compile time.
///
/// Like [`english!`], but for `petname::lang::Turkish`. The adjectives file
/// uses the same whitespace-delimited format, except a token may carry an
/// emphatic (reduplicated) form after an `=`, e.g. `kırmızı=kıpkırmızı`. The
/// adverbs and nouns files are plain whitespace-delimited words.
///
/// ```ignore
/// let t = petname::turkish!("words/turkish");
/// ```
#[proc_macro]
pub fn turkish(input: TokenStream) -> TokenStream {
    crate::lang::turkish::expand(input)
}

/// Alias for [`english!`].
#[proc_macro]
pub fn petnames(input: TokenStream) -> TokenStream {
    crate::lang::english::expand(input)
}
