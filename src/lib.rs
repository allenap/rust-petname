#![no_std]
//!
//! [`petname()`] will generate a single name with a default random number
//! generator:
//!
//! ```rust
//! # #[cfg(all(feature = "default-rng", feature = "default-words"))]
//! let name: Option<String> = petname::petname(3, "-");
//! // e.g. deftly-apt-swiftlet
//! ```
//!
//! You can bring your own random number generator from [rand][]:
//!
//! ```rust
//! # #[cfg(feature = "default-rng")] {
//! let mut rng = rand::rngs::ThreadRng::default();
//! # #[cfg(feature = "default-words")] {
//! let petnames = petname::Petnames::default();
//! let name = petnames.namer(7, ":").iter(&mut rng).next().expect("no names");
//! # } }
//! ```
//!
//! See that call to [`namer`][`Petnames::namer`] above? It returned a
//! [`Namer`]. Calling [`iter`][`Namer::iter`] on that gives a standard
//! [`Iterator`]. This is more efficient than calling [`petname()`] repeatedly,
//! plus you get all the features of Rust iterators:
//!
//! ```rust
//! # #[cfg(feature = "default-rng")]
//! let mut rng = rand::rngs::ThreadRng::default();
//! # #[cfg(feature = "default-words")]
//! let petnames = petname::Petnames::default();
//! # #[cfg(all(feature = "default-rng", feature = "default-words"))]
//! let ten_thousand_names: Vec<String> =
//!   petnames.namer(3, "_").iter(&mut rng).take(10000).collect();
//! ```
//!
//! 💡 Even more efficient but slightly less convenient is
//! [`Namer::generate_into`].
//!
//! # Word lists
//!
//! You can populate [`Petnames`] with your own word lists at runtime, but the
//! word lists from upstream [petname][] are included with the `default-words`
//! feature (which is enabled by default). See [`Petnames::small`],
//! [`Petnames::medium`], and [`Petnames::large`] to select a particular
//! built-in word list, or use [`Petnames::default`].
//!
//! ## Embedding your own word lists
//!
//! The [`petnames!`] macro will statically embed your own word lists at
//! compile-time. This is available with the `macros` feature (enabled by
//! default). This same mechanism is used to embed the default word lists.
//!
//! [petname]: https://github.com/dustinkirkland/petname
//!
//! ## Basic filtering
//!
//! You can modify the word lists to, for example, only use words beginning with
//! the letter "b":
//!
//! ```rust
//! # #[cfg(feature = "default-words")] {
//! let mut petnames = petname::Petnames::default();
//! petnames.retain(|s| s.starts_with("b"));
//! # #[cfg(feature = "default-rng")] {
//! let name = petnames.namer(3, ".").iter(&mut rand::rng()).next().expect("no names");
//! assert!(name.starts_with('b'));
//! # } }
//! ```
//!
//! ## Alliterating
//!
//! There is another way to generate alliterative petnames, useful in particular
//! when you don't need or want each name to be limited to using the same
//! initial letter as the previous generated name. Create the `Petnames` as
//! before, and then convert it into an [`Alliterations`]:
//!
//! ```rust
//! # #[cfg(feature = "default-words")] {
//! let mut petnames = petname::Petnames::default();
//! let mut alliterations: petname::Alliterations = petnames.into();
//! # #[cfg(feature = "default-rng")]
//! alliterations.namer(3, "/").iter(&mut rand::rng()).next().expect("no names");
//! # }
//! ```
//!
//! # The [`Generator`] trait
//!
//! Both [`Petnames`] and [`Alliterations`] implement [`Generator`]. It's
//! [object-safe] so you can use them as trait objects:
//!
//! [object-safe]:
//!     https://doc.rust-lang.org/reference/items/traits.html#object-safety
//!
//! ```rust
//! use petname::Generator;
//! let mut buf = String::new();
//! # #[cfg(all(feature = "default-words", feature = "default-rng"))] {
//! let petnames: &dyn Generator = &petname::Petnames::default();
//! petnames.generate_into(&mut buf, &mut rand::rng(), 3, "/");
//! let alliterations: &dyn Generator = &petname::Alliterations::default();
//! alliterations.generate_into(&mut buf, &mut rand::rng(), 3, "/");
//! # }
//! ```
//!

extern crate alloc;

#[cfg(feature = "macros")]
extern crate self as petname;

use alloc::{borrow::Cow, collections::BTreeMap, string::String, vec::Vec};

use rand::seq::{IndexedRandom, IteratorRandom};

/// Convenience function to generate a new petname from default word lists.
#[allow(dead_code)]
#[cfg(all(feature = "default-rng", feature = "default-words"))]
pub fn petname(words: u8, separator: &str) -> Option<String> {
    Petnames::default().namer(words, separator).iter(&mut rand::rng()).next()
}

/// A word list.
pub type Words<'a> = Cow<'a, [&'a str]>;

// Re-export proc macro.
#[cfg(feature = "macros")]
pub use petname_macros::petnames;

/// Trait that defines a generator of petnames, as consumed by [`Namer`].
///
/// The sole required method is [`generate_into`][`Self::generate_into`].
///
/// This trait is [object-safe] so you can use implementors as trait objects.
///
/// [object-safe]:
///     https://doc.rust-lang.org/reference/items/traits.html#object-safety
///
pub trait Generator {
    /// Generate a petname into a given [`String`] buffer.
    ///
    /// This method does not clear the buffer. The generated name is pushed at
    /// the end of the string. The name _may_ contain fewer words than requested
    /// if one or more of the word lists are empty.
    ///
    fn generate_into(&self, buf: &mut String, rng: &mut dyn rand::Rng, words: u8, separator: &str);
}

/// A configured petname generator.
///
/// Created by [`Petnames::namer`] or [`Alliterations::namer`]. Holds a
/// reference to a word list, a word count, and a separator. Call
/// [`iter`][`Self::iter`] to get an [`Iterator`] over generated names, or
/// [`generate_into`][`Self::generate_into`] to write into a buffer directly.
///
/// # Examples
///
/// ```rust
/// # #[cfg(all(feature = "default-rng", feature = "default-words"))] {
/// let petnames = petname::Petnames::default();
/// let namer = petnames.namer(3, "-");
///
/// // As an iterator:
/// let names: Vec<String> = namer.iter(&mut rand::rng()).take(10).collect();
///
/// // Or writing into a buffer:
/// let mut buf = String::new();
/// namer.generate_into(&mut buf, &mut rand::rng());
/// # }
/// ```
///
pub struct Namer<'a, G: ?Sized> {
    generator: &'a G,
    words: u8,
    separator: &'a str,
}

impl<'a, G: Generator + ?Sized> Namer<'a, G> {
    /// Generate a petname into a given [`String`] buffer.
    ///
    /// This can be more efficient than [`iter`][`Self::iter`] when generating
    /// many names because the buffer can be reused; each name yielded by
    /// [`iter`][`Self::iter`] allocates a new `String`.
    ///
    /// This method does not clear the buffer. The generated name is pushed at
    /// the end of the string.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #[cfg(all(feature = "default-rng", feature = "default-words"))] {
    /// let petnames = petname::Petnames::default();
    /// let namer = petnames.namer(7, "::");
    /// let mut buf = String::new();
    /// namer.generate_into(&mut buf, &mut rand::rng());
    /// assert_eq!(7, buf.split("::").count());
    /// # }
    /// ```
    ///
    /// When looping you might want to check if the buffer has been modified or
    /// not. An unmodified buffer might mean that the source of names or
    /// randomness has been exhausted.
    ///
    /// ```rust
    /// # #[cfg(all(feature = "default-rng", feature = "default-words"))] {
    /// let petnames = petname::Petnames::default();
    /// let namer = petnames.namer(3, "+");
    /// let mut buf = String::new();
    /// loop {
    ///     namer.generate_into(&mut buf, &mut rand::rng());
    ///     if buf.is_empty() {
    ///         break;  // Source exhausted?
    ///     } else {
    ///         println!("Petname: {buf}");
    ///         buf.clear();  // Reset before next iteration.
    ///         # break;
    ///     }
    /// }
    /// # }
    /// ```
    ///
    pub fn generate_into(&self, buf: &mut String, rng: &mut dyn rand::Rng) {
        self.generator.generate_into(buf, rng, self.words, self.separator);
    }

    /// Iterator yielding petnames.
    ///
    /// Note that a new [`String`] is allocated for each name yielded. If this
    /// is a problem, consider [`generate_into`][`Self::generate_into`] instead.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #[cfg(all(feature = "default-rng", feature = "default-words"))] {
    /// let petnames = petname::Petnames::default();
    /// let mut rng = rand::rngs::ThreadRng::default();
    /// let mut namer = petnames.namer(4, "_");
    /// println!("name: {}", namer.iter(&mut rng).next().unwrap());
    /// # }
    /// ```
    pub fn iter<'b>(&'b self, rng: &'b mut dyn rand::Rng) -> impl Iterator<Item = String> + 'b {
        core::iter::from_fn(move || {
            let mut buf = String::new();
            self.generate_into(&mut buf, rng);
            (!buf.is_empty()).then_some(buf)
        })
    }
}

/// Word lists and the logic to combine them into _petnames_.
///
/// A _petname_ with `n` words will contain, in order:
///
///   * `n - 2` adverbs when `n >= 2`, otherwise 0 adverbs.
///   * 1 adjective when `n >= 2`, otherwise 0 adjectives.
///   * 1 noun when `n >= 1`, otherwise 0 nouns.
///
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Petnames<'a> {
    pub adjectives: Words<'a>,
    pub adverbs: Words<'a>,
    pub nouns: Words<'a>,
}

impl<'a> Petnames<'a> {
    /// Constructs a new `Petnames` from the small word lists.
    #[cfg(feature = "default-words")]
    pub fn small() -> Self {
        petnames!("words/small")
    }

    /// Constructs a new `Petnames` from the medium word lists.
    #[cfg(feature = "default-words")]
    pub fn medium() -> Self {
        petnames!("words/medium")
    }

    /// Constructs a new `Petnames` from the large word lists.
    #[cfg(feature = "default-words")]
    pub fn large() -> Self {
        petnames!("words/large")
    }

    /// Constructs a new `Petnames` from the given word lists.
    ///
    /// The words are extracted from the given strings by splitting on whitespace.
    pub fn new(adjectives: &'a str, adverbs: &'a str, nouns: &'a str) -> Self {
        Self {
            adjectives: Cow::Owned(adjectives.split_whitespace().collect()),
            adverbs: Cow::Owned(adverbs.split_whitespace().collect()),
            nouns: Cow::Owned(nouns.split_whitespace().collect()),
        }
    }

    /// Keep words matching a predicate.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #[cfg(feature = "default-words")] {
    /// let mut petnames = petname::Petnames::default();
    /// petnames.retain(|s| s.starts_with("h"));
    /// # #[cfg(feature = "default-rng")]
    /// assert!(petnames.namer(2, ".").iter(&mut rand::rng()).next().unwrap().starts_with('h'));
    /// # }
    /// ```
    ///
    /// This is a convenience wrapper that applies the same predicate to the
    /// adjectives, adverbs, and nouns lists.
    ///
    pub fn retain<F>(&mut self, mut predicate: F)
    where
        F: FnMut(&str) -> bool,
    {
        self.adjectives.to_mut().retain(|word| predicate(word));
        self.adverbs.to_mut().retain(|word| predicate(word));
        self.nouns.to_mut().retain(|word| predicate(word));
    }

    /// Calculate the cardinality of this `Petnames`.
    ///
    /// If this is low, names may be repeated by the generator with a higher
    /// frequency than your use-case may allow.
    ///
    /// This can saturate. If the total possible combinations of words exceeds
    /// `u128::MAX` then this will return `u128::MAX`.
    pub fn cardinality(&self, words: u8) -> u128 {
        Lists::new(words)
            .map(|list| match list {
                List::Adverb => self.adverbs.len() as u128,
                List::Adjective => self.adjectives.len() as u128,
                List::Noun => self.nouns.len() as u128,
            })
            .reduce(u128::saturating_mul)
            .unwrap_or(0u128)
    }

    /// Create a [`Namer`] that generates petnames from these word lists.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #[cfg(all(feature = "default-rng", feature = "default-words"))]
    /// let name = petname::Petnames::default()
    ///     .namer(3, "-")
    ///     .iter(&mut rand::rng())
    ///     .next()
    ///     .expect("no names");
    /// ```
    pub fn namer<'b>(&'b self, words: u8, separator: &'b str) -> Namer<'b, Self> {
        Namer { generator: self, words, separator }
    }
}

impl Generator for Petnames<'_> {
    fn generate_into(&self, buf: &mut String, rng: &mut dyn rand::Rng, words: u8, separator: &str) {
        for list in Lists::new(words) {
            match list {
                List::Adverb => {
                    if let Some(word) = self.adverbs.choose(rng).copied() {
                        buf.push_str(word);
                        buf.push_str(separator);
                    }
                }
                List::Adjective => {
                    if let Some(word) = self.adjectives.choose(rng).copied() {
                        buf.push_str(word);
                        buf.push_str(separator);
                    }
                }
                List::Noun => {
                    if let Some(word) = self.nouns.choose(rng).copied() {
                        buf.push_str(word);
                    }
                }
            };
        }
    }
}

#[cfg(feature = "default-words")]
impl Default for Petnames<'_> {
    /// Constructs a new [`Petnames`] from the default (medium) word lists.
    fn default() -> Self {
        Self::medium()
    }
}

/// Word lists prepared for alliteration.
///
/// Construct from a [`Petnames`] with [`Alliterations::from`]. This takes that
/// instance and splits it into several _groups_. In each, all of the nouns,
/// adverbs, and adjectives will start with the same letter. A name generated
/// from any of them will naturally produce an alliterative petname.
///
/// You can also create one of these from an iterable of `(char, Petnames)`.
/// This might be useful for testing, or for repurposing this to generate names
/// with assonance, say.
///
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Alliterations<'a> {
    groups: BTreeMap<char, Petnames<'a>>,
}

impl Alliterations<'_> {
    /// Keep only those groups that match a predicate.
    ///
    /// A _group_ is defined by a [`char`] and a corresponding [`Petnames`]
    /// instance.
    ///
    /// The given predicate can return `true` to keep the group or `false` to
    /// evict it. It can also mutate each `Petnames` instance. The notional
    /// invariant is that every noun, adverb, and adjective in that `Petnames`
    /// instance should start with that `char`, but it's okay to break that.
    ///
    pub fn retain<F>(&mut self, predicate: F)
    where
        F: FnMut(&char, &mut Petnames) -> bool,
    {
        self.groups.retain(predicate)
    }

    /// Calculate the cardinality of this `Alliterations`.
    ///
    /// This is the sum of the cardinality of all groups.
    ///
    /// This can saturate. If the total possible combinations of words exceeds
    /// `u128::MAX` then this will return `u128::MAX`.
    pub fn cardinality(&self, words: u8) -> u128 {
        self.groups
            .values()
            .map(|petnames| petnames.cardinality(words))
            .reduce(u128::saturating_add)
            .unwrap_or(0u128)
    }

    /// Create a [`Namer`] that generates alliterative petnames from these word
    /// lists.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #[cfg(all(feature = "default-rng", feature = "default-words"))]
    /// let name = petname::Alliterations::default()
    ///     .namer(3, "-")
    ///     .iter(&mut rand::rng())
    ///     .next()
    ///     .expect("no names");
    /// ```
    pub fn namer<'b>(&'b self, words: u8, separator: &'b str) -> Namer<'b, Self> {
        Namer { generator: self, words, separator }
    }
}

impl<'a> From<Petnames<'a>> for Alliterations<'a> {
    fn from(petnames: Petnames<'a>) -> Self {
        let mut adjectives: BTreeMap<char, Vec<&str>> = group_words_by_first_letter(petnames.adjectives);
        let mut adverbs: BTreeMap<char, Vec<&str>> = group_words_by_first_letter(petnames.adverbs);
        let nouns: BTreeMap<char, Vec<&str>> = group_words_by_first_letter(petnames.nouns);
        // We find all adjectives and adverbs that start with the same letter as
        // each group of nouns. We start from nouns because it's possible to
        // have a petname with length of 1, i.e. a noun. This means that it's
        // okay at this point for the adjectives and adverbs lists to be empty.
        Alliterations {
            groups: nouns.into_iter().fold(BTreeMap::default(), |mut acc, (first_letter, nouns)| {
                acc.insert(
                    first_letter,
                    Petnames {
                        adjectives: adjectives.remove(&first_letter).unwrap_or_default().into(),
                        adverbs: adverbs.remove(&first_letter).unwrap_or_default().into(),
                        nouns: Cow::from(nouns),
                    },
                );
                acc
            }),
        }
    }
}

impl<'a, GROUPS> From<GROUPS> for Alliterations<'a>
where
    GROUPS: IntoIterator<Item = (char, Petnames<'a>)>,
{
    fn from(groups: GROUPS) -> Self {
        Self { groups: groups.into_iter().collect() }
    }
}

fn group_words_by_first_letter(words: Words<'_>) -> BTreeMap<char, Vec<&str>> {
    words.iter().fold(BTreeMap::default(), |mut acc, s| match s.chars().next() {
        Some(first_letter) => {
            acc.entry(first_letter).or_default().push(s);
            acc
        }
        None => acc,
    })
}

impl Generator for Alliterations<'_> {
    fn generate_into(&self, buf: &mut String, rng: &mut dyn rand::Rng, words: u8, separator: &str) {
        if let Some(group) = self.groups.values().choose(rng) {
            group.generate_into(buf, rng, words, separator);
        }
    }
}

#[cfg(feature = "default-words")]
impl Default for Alliterations<'_> {
    /// Constructs a new [`Alliterations`] from the default [`Petnames`].
    fn default() -> Self {
        Petnames::default().into()
    }
}

/// Enum representing which word list to use.
#[derive(Debug, PartialEq)]
enum List {
    Adverb,
    Adjective,
    Noun,
}

/// Iterator, yielding which word list to use next.
///
/// This yields the appropriate list – [adverbs][List::Adverb],
/// [adjectives][List::Adjective], [nouns][List::Noun] – from which to select a
/// word when constructing a petname of `n` words. For example, if you want 4
/// words in your petname, this will first yield [List::Adverb], then
/// [List::Adverb] again, then [List::Adjective], and lastly [List::Noun].
#[derive(Debug, PartialEq)]
enum Lists {
    Adverb(u8),
    Adjective,
    Noun,
    Done,
}

impl Lists {
    fn new(words: u8) -> Self {
        match words {
            0 => Self::Done,
            1 => Self::Noun,
            2 => Self::Adjective,
            n => Self::Adverb(n - 3),
        }
    }

    fn current(&self) -> Option<List> {
        match self {
            Self::Adjective => Some(List::Adjective),
            Self::Adverb(_) => Some(List::Adverb),
            Self::Noun => Some(List::Noun),
            Self::Done => None,
        }
    }

    fn advance(&mut self) {
        *self = match self {
            Self::Adverb(0) => Self::Adjective,
            Self::Adverb(remaining) => Self::Adverb(*remaining - 1),
            Self::Adjective => Self::Noun,
            Self::Noun | Self::Done => Self::Done,
        }
    }

    fn remaining(&self) -> usize {
        match self {
            Self::Adverb(n) => (n + 3) as usize,
            Self::Adjective => 2,
            Self::Noun => 1,
            Self::Done => 0,
        }
    }
}

impl Iterator for Lists {
    type Item = List;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.current();
        self.advance();
        current
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.remaining();
        (remaining, Some(remaining))
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn lists_sequences_adverbs_adjectives_then_names() {
        let mut lists = super::Lists::new(4);
        assert_eq!(super::Lists::Adverb(1), lists);
        assert_eq!(Some(super::List::Adverb), lists.next());
        assert_eq!(super::Lists::Adverb(0), lists);
        assert_eq!(Some(super::List::Adverb), lists.next());
        assert_eq!(super::Lists::Adjective, lists);
        assert_eq!(Some(super::List::Adjective), lists.next());
        assert_eq!(super::Lists::Noun, lists);
        assert_eq!(Some(super::List::Noun), lists.next());
        assert_eq!(super::Lists::Done, lists);
        assert_eq!(None, lists.next());
    }

    #[test]
    fn lists_size_hint() {
        let mut lists = super::Lists::new(3);
        assert_eq!((3, Some(3)), lists.size_hint());
        assert!(lists.next().is_some());
        assert_eq!((2, Some(2)), lists.size_hint());
        assert!(lists.next().is_some());
        assert_eq!((1, Some(1)), lists.size_hint());
        assert!(lists.next().is_some());
        assert_eq!((0, Some(0)), lists.size_hint());
        assert_eq!(None, lists.next());
        assert_eq!((0, Some(0)), lists.size_hint());
    }
}
