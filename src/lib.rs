#![no_std]
//!
//! You can populate [`Petnames`] with your own word lists, but the word lists
//! from upstream [petname](https://github.com/dustinkirkland/petname) are
//! included with the `default-words` feature (enabled by default). See
//! [`Petnames::small`], [`Petnames::medium`], and [`Petnames::large`] to select
//! a particular built-in word list, or use the [`Default`] implementation.
//!
//! The other thing you need is a random number generator from [rand][]:
//!
//! ```rust
//! # use petname::Generator;
//! # #[cfg(feature = "default-rng")]
//! let mut rng = rand::thread_rng();
//! # #[cfg(all(feature = "default-rng", feature = "default-words"))]
//! let pname = petname::Petnames::default().generate(&mut rng, 7, ":");
//! ```
//!
//! It may be more convenient to use the default random number generator:
//!
//! ```rust
//! # use petname::Generator;
//! # #[cfg(all(feature = "default-rng", feature = "default-words"))]
//! let pname = petname::Petnames::default().generate_one(7, ":");
//! ```
//!
//! There's a [convenience function][petname] that'll do all of this:
//!
//! ```rust
//! # #[cfg(all(feature = "default-rng", feature = "default-words"))]
//! let pname = petname::petname(7, ":");
//! ```
//!
//! But the most flexible approach is to create an [`Iterator`] with
//! [`iter`][`Petnames::iter`]:
//!
//! ```rust
//! # use petname::Generator;
//! # #[cfg(feature = "default-rng")]
//! let mut rng = rand::thread_rng();
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
//! # #[cfg(feature = "default-words")]
//! let mut petnames = petname::Petnames::default();
//! # #[cfg(feature = "default-words")]
//! petnames.retain(|s| s.starts_with("b"));
//! # #[cfg(all(feature = "default-rng", feature = "default-words"))]
//! petnames.generate_one(3, ".");
//! ```
//!

extern crate alloc;

use alloc::{
    borrow::Cow,
    boxed::Box,
    collections::BTreeMap,
    string::{String, ToString},
    vec::Vec,
};

use itertools::Itertools;
use rand::seq::{IteratorRandom, SliceRandom};

/// Convenience function to generate a new petname from default word lists.
#[allow(dead_code)]
#[cfg(all(feature = "default-rng", feature = "default-words"))]
pub fn petname(words: u8, separator: &str) -> Option<String> {
    Petnames::default().generate_one(words, separator)
}

/// A word list.
pub type Words<'a> = Cow<'a, [&'a str]>;

#[cfg(feature = "default-words")]
mod words {
    include!(concat!(env!("OUT_DIR"), "/words.rs"));
}

/// Trait that defines a generator of petnames.
///
/// There are default implementations of `generate_one` and `iter`, i.e. only
/// `generate` needs to be implemented.
///
pub trait Generator<'a> {
    /// Generate a new petname.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use petname::Generator;
    /// # #[cfg(all(feature = "default-rng", feature = "default-words"))]
    /// let mut rng = rand::thread_rng();
    /// # #[cfg(all(feature = "default-rng", feature = "default-words"))]
    /// petname::Petnames::default().generate(&mut rng, 7, ":");
    /// ```
    ///
    /// # Notes
    ///
    /// This may return fewer words than you request if one or more of the word
    /// lists are empty. For example, if there are no adverbs, requesting 3 or
    /// more words may still yield only "doubtful-salmon".
    ///
    fn generate<RNG>(&self, rng: &mut RNG, words: u8, separator: &str) -> Option<String>
    where
        RNG: rand::Rng;

    /// Generate a single new petname.
    ///
    /// This is like `generate` but uses `rand::thread_rng` as the random
    /// source. For efficiency use `generate` when creating multiple names, or
    /// when you want to use a custom source of randomness.
    #[cfg(feature = "default-rng")]
    fn generate_one(&self, words: u8, separator: &str) -> Option<String> {
        self.generate(&mut rand::thread_rng(), words, separator)
    }

    /// Iterator yielding petnames.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use petname::Generator;
    /// # #[cfg(all(feature = "default-rng", feature = "default-words"))]
    /// let mut rng = rand::thread_rng();
    /// # #[cfg(all(feature = "default-rng", feature = "default-words"))]
    /// let petnames = petname::Petnames::default();
    /// # #[cfg(all(feature = "default-rng", feature = "default-words"))]
    /// let mut iter = petnames.iter(&mut rng, 4, "_");
    /// # #[cfg(all(feature = "default-rng", feature = "default-words"))]
    /// println!("name: {}", iter.next().unwrap());
    /// ```
    fn iter<RNG>(
        &'a self,
        rng: &'a mut RNG,
        words: u8,
        separator: &str,
    ) -> Box<dyn Iterator<Item = String> + 'a>
    where
        RNG: rand::Rng,
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
        Self {
            adjectives: Cow::from(&words::small::ADJECTIVES[..]),
            adverbs: Cow::from(&words::small::ADVERBS[..]),
            nouns: Cow::from(&words::small::NOUNS[..]),
        }
    }

    /// Constructs a new `Petnames` from the medium word lists.
    #[cfg(feature = "default-words")]
    pub fn medium() -> Self {
        Self {
            adjectives: Cow::from(&words::medium::ADJECTIVES[..]),
            adverbs: Cow::from(&words::medium::ADVERBS[..]),
            nouns: Cow::from(&words::medium::NOUNS[..]),
        }
    }

    /// Constructs a new `Petnames` from the large word lists.
    #[cfg(feature = "default-words")]
    pub fn large() -> Self {
        Self {
            adjectives: Cow::from(&words::large::ADJECTIVES[..]),
            adverbs: Cow::from(&words::large::ADVERBS[..]),
            nouns: Cow::from(&words::large::NOUNS[..]),
        }
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
    /// # #[cfg(feature = "default-words")]
    /// let mut petnames = petname::Petnames::default();
    /// # #[cfg(feature = "default-words")]
    /// petnames.retain(|s| s.starts_with("b"));
    /// # #[cfg(feature = "default-words")]
    /// # #[cfg(feature = "default-rng")]
    /// petnames.generate_one(2, ".");
    /// ```
    ///
    /// This is merely a convenience wrapper that applies the same predicate to
    /// the adjectives, adverbs, and nouns lists.
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
    fn generate<RNG>(&self, rng: &mut RNG, words: u8, separator: &str) -> Option<String>
    where
        RNG: rand::Rng,
    {
        let name = Itertools::intersperse(
            Lists::new(words).filter_map(|list| match list {
                List::Adverb => self.adverbs.choose(rng).copied(),
                List::Adjective => self.adjectives.choose(rng).copied(),
                List::Noun => self.nouns.choose(rng).copied(),
            }),
            separator,
        )
        .collect::<String>();
        if name.is_empty() {
            None
        } else {
            Some(name)
        }
    }
}

#[cfg(feature = "default-words")]
impl<'a> Default for Petnames<'a> {
    /// Constructs a new `Petnames` from the default (small) word lists.
    fn default() -> Self {
        Self::small()
    }
}

/// Word lists prepared for alliteration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Alliterations<'a> {
    pub groups: BTreeMap<char, Petnames<'a>>,
}

impl<'a> Alliterations<'a> {
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

fn group_words_by_first_letter(words: Words) -> BTreeMap<char, Vec<&str>> {
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
    /// # #[cfg(all(feature = "default-rng", feature = "default-words"))]
    /// let mut rng = rand::thread_rng();
    /// # #[cfg(all(feature = "default-rng", feature = "default-words"))]
    /// petname::Petnames::default().generate(&mut rng, 7, ":");
    /// ```
    ///
    /// # Notes
    ///
    /// This may return fewer words than you request if one or more of the word
    /// lists are empty. For example, if there are no adverbs, requesting 3 or
    /// more words may still yield only "doubtful-salmon".
    ///
    fn generate<RNG>(&self, rng: &mut RNG, words: u8, separator: &str) -> Option<String>
    where
        RNG: rand::Rng,
    {
        self.groups.values().choose(rng).and_then(|group| group.generate(rng, words, separator))
    }
}

#[cfg(feature = "default-words")]
impl<'a> Default for Alliterations<'a> {
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
/// [adjectives][List::Adjective]s, [nouns][List::Nouns] –  from which to select
/// a word when constructing a petname of `n` words. For example, if you want 4
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
struct Names<'a, RNG, GENERATOR>
where
    RNG: rand::Rng,
    GENERATOR: Generator<'a>,
{
    generator: &'a GENERATOR,
    rng: &'a mut RNG,
    words: u8,
    separator: String,
}

impl<'a, RNG, GENERATOR> Iterator for Names<'a, RNG, GENERATOR>
where
    RNG: rand::Rng,
    GENERATOR: Generator<'a>,
{
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        self.generator.generate(self.rng, self.words, &self.separator)
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
