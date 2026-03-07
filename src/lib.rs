#![no_std]
//!
//! You can populate [`Petnames`] with your own word lists, but the word lists
//! from upstream [petname](https://github.com/dustinkirkland/petname) are
//! included with the `default-words` feature (enabled by default). See
//! [`Petnames::small`], [`Petnames::medium`], and [`Petnames::large`] to select
//! a particular built-in word list, or use [`Petnames::default`].
//!
//! For more flexibility, see the [`petnames!`] macro, with which you can
//! statically embed your own word lists at compile-time. This is available with
//! the `macros` feature (enabled by default). The same mechanism is used to
//! embed the default word lists.
//!
//! To generate a petname, the other thing you need is a random number generator
//! from [rand][]:
//!
//! ```rust
//! use petname::Generator; // Trait needs to be in scope for `generate`.
//! # #[cfg(feature = "default-rng")]
//! let mut rng = rand::rngs::ThreadRng::default();
//! # #[cfg(all(feature = "default-rng", feature = "default-words"))]
//! let name = petname::Petnames::default().iter(&mut rng, 7, ":").next().expect("no names");
//! ```
//!
//! There's a [convenience function][petname()] that'll generate a single name
//! with a default random number generator:
//!
//! ```rust
//! # #[cfg(all(feature = "default-rng", feature = "default-words"))]
//! let name = petname::petname(7, ":");
//! ```
//!
//! A more flexible approach is to create an [`Iterator`] with
//! [`iter`][`Petnames::iter`]:
//!
//! ```rust
//! use petname::Generator; // Trait needs to be in scope for `iter`.
//! # #[cfg(feature = "default-rng")]
//! let mut rng = rand::rngs::ThreadRng::default();
//! # #[cfg(feature = "default-words")]
//! let petnames = petname::Petnames::default();
//! # #[cfg(all(feature = "default-rng", feature = "default-words"))]
//! let ten_thousand_names: Vec<String> =
//!   petnames.iter(&mut rng, 3, "_").take(10000).collect();
//! ```
//!
//! You can modify the word lists to, for example, only use words beginning with
//! the letter "b":
//!
//! ```rust
//! # use petname::Generator;
//! # #[cfg(feature = "default-words")] {
//! let mut petnames = petname::Petnames::default();
//! petnames.retain(|s| s.starts_with("b"));
//! # #[cfg(feature = "default-rng")] {
//! let name = petnames.iter(&mut rand::rng(), 3, ".").next().expect("no names");
//! assert!(name.starts_with('b'));
//! # } }
//! ```
//!
//! There's another way to generate alliterative petnames which is useful when
//! you don't need or want each name to be limited to using the same initial
//! letter as the previous generated name. Create the `Petnames` as before, and
//! then convert it into an [`Alliterations`]:
//!
//! ```rust
//! # use petname::Generator;
//! # #[cfg(feature = "default-words")] {
//! let mut petnames = petname::Petnames::default();
//! let mut alliterations: petname::Alliterations = petnames.into();
//! # #[cfg(feature = "default-rng")]
//! alliterations.iter(&mut rand::rng(), 3, "/").next().expect("no names");
//! # }
//! ```
//!
//! Both [`Petnames`] and [`Alliterations`] implement [`Generator`]; this needs
//! to be in scope in order to generate names. It's [object-safe] so you can use
//! `Petnames` and `Alliterations` as trait objects:
//!
//! [object-safe]:
//!     https://doc.rust-lang.org/reference/items/traits.html#object-safety
//!
//! ```rust
//! use petname::Generator;
//! # #[cfg(feature = "default-words")] {
//! let generator: &dyn Generator = &petname::Petnames::default();
//! let generator: &dyn Generator = &petname::Alliterations::default();
//! # }
//! ```
//!

extern crate alloc;

#[cfg(feature = "macros")]
extern crate self as petname;

use alloc::{
    borrow::Cow,
    boxed::Box,
    collections::BTreeMap,
    string::{String, ToString},
    vec::Vec,
};

use rand::seq::{IndexedRandom, IteratorRandom};

/// Convenience function to generate a new petname from default word lists.
#[allow(dead_code)]
#[cfg(all(feature = "default-rng", feature = "default-words"))]
pub fn petname(words: u8, separator: &str) -> Option<String> {
    Petnames::default().iter(&mut rand::rng(), words, separator).next()
}

/// A word list.
pub type Words<'a> = Cow<'a, [&'a str]>;

// Re-export proc macro.
#[cfg(feature = "macros")]
pub use petname_macros::petnames;

/// Trait that defines a generator of petnames.
///
/// There is a default implementation of [`iter`][`Self::iter`]; only
/// [`generate_parts`][`Self::generate_parts`] needs to be implemented.
///
pub trait Generator<'a> {
    /// Generate the parts for a new petname.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use petname::Generator;
    /// let mut buf: Vec<&str> = Vec::with_capacity(7);
    /// # #[cfg(all(feature = "default-rng", feature = "default-words"))] {
    /// let mut rng = rand::rngs::ThreadRng::default();
    /// petname::Petnames::default().generate_parts(&mut buf, &mut rng, 7);
    /// assert_eq!(7, buf.len());
    /// # }
    /// ```
    ///
    /// # Notes
    ///
    /// This may return fewer words than you request if one or more of the word
    /// lists are empty. For example, if there are no adverbs, requesting 3 or
    /// more words may still yield only `["doubtful", "salmon"]`.
    ///
    fn generate_parts(&self, buf: &mut Vec<&'a str>, rng: &mut dyn rand::Rng, words: u8);

    /// Iterator yielding petnames.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use petname::Generator;
    /// # #[cfg(all(feature = "default-rng", feature = "default-words"))] {
    /// let mut rng = rand::rngs::ThreadRng::default();
    /// let petnames = petname::Petnames::default();
    /// let mut iter = petnames.iter(&mut rng, 4, "_");
    /// println!("name: {}", iter.next().unwrap());
    /// # }
    /// ```
    fn iter(
        &'a self,
        rng: &'a mut dyn rand::Rng,
        words: u8,
        separator: &str,
    ) -> Box<dyn Iterator<Item = String> + 'a>
    where
        Self: Sized,
    {
        let names = Names { generator: self, rng, words, separator: separator.to_string() };
        Box::new(names)
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
    /// # use petname::Generator;
    /// # #[cfg(feature = "default-words")] {
    /// let mut petnames = petname::Petnames::default();
    /// petnames.retain(|s| s.starts_with("h"));
    /// # #[cfg(feature = "default-rng")]
    /// assert!(petnames.iter(&mut rand::rng(), 2, ".").next().unwrap().starts_with('h'));
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
}

impl<'a> Generator<'a> for Petnames<'a> {
    fn generate_parts(&self, buf: &mut Vec<&'a str>, rng: &mut dyn rand::Rng, words: u8) {
        buf.extend(Lists::new(words).filter_map(|list| match list {
            List::Adverb => self.adverbs.choose(rng).copied(),
            List::Adjective => self.adjectives.choose(rng).copied(),
            List::Noun => self.nouns.choose(rng).copied(),
        }));
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
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Alliterations<'a> {
    groups: BTreeMap<char, Petnames<'a>>,
}

impl Alliterations<'_> {
    /// Keep only those groups that match a predicate.
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

impl<'a> Generator<'a> for Alliterations<'a> {
    /// Generate a new petname.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use petname::Generator;
    /// # #[cfg(all(feature = "default-rng", feature = "default-words"))] {
    /// let mut rng = rand::rngs::ThreadRng::default();
    /// petname::Petnames::default().iter(&mut rng, 7, ":").next();
    /// # }
    /// ```
    ///
    /// # Notes
    ///
    /// This may return fewer words than you request if one or more of the word
    /// lists are empty. For example, if there are no adverbs, requesting 3 or
    /// more words may still yield only "doubtful-salmon".
    ///
    fn generate_parts(&self, buf: &mut Vec<&'a str>, rng: &mut dyn rand::Rng, words: u8) {
        if let Some(group) = self.groups.values().choose(rng) {
            group.generate_parts(buf, rng, words);
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

/// Iterator yielding petnames.
struct Names<'a, GENERATOR>
where
    GENERATOR: Generator<'a>,
{
    generator: &'a GENERATOR,
    rng: &'a mut dyn rand::Rng,
    words: u8,
    separator: String,
}

impl<'a, GENERATOR> Iterator for Names<'a, GENERATOR>
where
    GENERATOR: Generator<'a>,
{
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        let mut parts = Vec::with_capacity(self.words as usize);
        self.generator.generate_parts(&mut parts, self.rng, self.words);
        let mut parts = parts.iter();
        if let Some(first) = parts.next() {
            let mut buf = String::new();
            buf.push_str(first);
            for part in parts {
                buf.push_str(&self.separator);
                buf.push_str(part);
            }
            Some(buf)
        } else {
            None
        }
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
