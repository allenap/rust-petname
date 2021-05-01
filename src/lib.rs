#![no_std]

extern crate alloc;

use alloc::{
    string::{String, ToString},
    vec::Vec,
};

use itertools::Itertools;
use rand::seq::SliceRandom;

/// Convenience function to generate a new petname from default word lists.
#[allow(dead_code)]
#[cfg(feature = "std_rng")]
#[cfg(feature = "default_dictionary")]
pub fn petname(words: u8, separator: &str) -> String {
    Petnames::new().generate_one(words, separator)
}

/// A word list.
pub type Words<'a> = Vec<&'a str>;

/// Word lists and the logic to combine them into _petnames_.
///
/// A _petname_ with `n` words will contain, in order:
///
///   * `n - 2` adverbs when `n >= 2`, otherwise 0 adverbs.
///   * 1 adjective when `n >= 2`, otherwise 0 adjectives.
///   * 1 name / noun when `n >= 1`, otherwise 0 names.
///
#[derive(Clone, Debug, PartialEq)]
pub struct Petnames<'a> {
    pub adjectives: Words<'a>,
    pub adverbs: Words<'a>,
    pub names: Words<'a>,
}

impl<'a> Petnames<'a> {
    /// Constructs a new `Petnames` from the default (small) word lists.
    #[cfg(feature = "default_dictionary")]
    pub fn new() -> Self {
        Self::default()
    }

    /// Constructs a new `Petnames` from the small word lists.
    #[cfg(feature = "default_dictionary")]
    pub fn small() -> Self {
        Self::init(
            include_str!("../words/small/adjectives.txt"),
            include_str!("../words/small/adverbs.txt"),
            include_str!("../words/small/names.txt"),
        )
    }

    /// Constructs a new `Petnames` from the medium word lists.
    #[cfg(feature = "default_dictionary")]
    pub fn medium() -> Self {
        Self::init(
            include_str!("../words/medium/adjectives.txt"),
            include_str!("../words/medium/adverbs.txt"),
            include_str!("../words/medium/names.txt"),
        )
    }

    /// Constructs a new `Petnames` from the large word lists.
    #[cfg(feature = "default_dictionary")]
    pub fn large() -> Self {
        Self::init(
            include_str!("../words/large/adjectives.txt"),
            include_str!("../words/large/adverbs.txt"),
            include_str!("../words/large/names.txt"),
        )
    }

    /// Constructs a new `Petnames` from the given word lists.
    ///
    /// The words are extracted from the given strings by splitting on whitespace.
    pub fn init(adjectives: &'a str, adverbs: &'a str, names: &'a str) -> Self {
        Self {
            adjectives: adjectives.split_whitespace().collect(),
            adverbs: adverbs.split_whitespace().collect(),
            names: names.split_whitespace().collect(),
        }
    }

    /// Keep words matching a predicate.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #[cfg(feature = "default_dictionary")]
    /// let mut petnames = petname::Petnames::default();
    /// # #[cfg(feature = "default_dictionary")]
    /// petnames.retain(|s| s.starts_with("b"));
    /// # #[cfg(feature = "default_dictionary")]
    /// # #[cfg(feature = "std_rng")]
    /// petnames.generate_one(2, ".");
    /// ```
    ///
    /// This is merely a convenience wrapper that applies the same predicate to
    /// the adjectives, adverbs, and names lists.
    ///
    pub fn retain<F>(&mut self, mut predicate: F)
    where
        F: FnMut(&str) -> bool,
    {
        self.adjectives.retain(|word| predicate(word));
        self.adverbs.retain(|word| predicate(word));
        self.names.retain(|word| predicate(word));
    }

    /// Calculate the cardinality of this `Petnames`.
    ///
    /// If this is low, names may be repeated by the generator with a higher
    /// frequency than your use-case may allow. If it is 0 (zero) the generator
    /// will panic (unless `words` is also zero).
    ///
    /// This can saturate. If the total possible combinations of words exceeds
    /// `u128::MAX` then this will return `u128::MAX`.
    pub fn cardinality(&self, words: u8) -> u128 {
        Lists(self, words)
            .map(|list| list.len() as u128)
            .fold1(u128::saturating_mul)
            .unwrap_or(0u128)
    }

    /// Generate a new petname.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #[cfg(all(feature = "std_rng", feature = "default_dictionary"))]
    /// let mut rng = rand::thread_rng();
    /// # #[cfg(all(feature = "std_rng", feature = "default_dictionary"))]
    /// petname::Petnames::default().generate(&mut rng, 7, ":");
    /// ```
    ///
    /// # Notes
    ///
    /// This may return fewer words than you request if one or more of the word
    /// lists are empty. For example, if there are no adverbs, requesting 3 or
    /// more words may still yield only "doubtful-salmon".
    ///
    pub fn generate<RNG>(&self, rng: &mut RNG, words: u8, separator: &str) -> String
    where
        RNG: rand::Rng,
    {
        itertools::Itertools::intersperse(
            Lists(self, words)
                .filter_map(|list| list.choose(rng))
                .cloned(),
            separator,
        )
        .collect::<String>()
    }

    /// Generate a single new petname.
    ///
    /// This is like `generate` but uses `rand::thread_rng` as the random
    /// source. For efficiency use `generate` when creating multiple names, or
    /// when you want to use a custom source of randomness.
    #[cfg(feature = "std_rng")]
    pub fn generate_one(&self, words: u8, separator: &str) -> String {
        self.generate(&mut rand::thread_rng(), words, separator)
    }

    /// Iterator yielding petnames.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #[cfg(all(feature = "std_rng", feature = "default_dictionary"))]
    /// let mut rng = rand::thread_rng();
    /// # #[cfg(all(feature = "std_rng", feature = "default_dictionary"))]
    /// let petnames = petname::Petnames::default();
    /// # #[cfg(all(feature = "std_rng", feature = "default_dictionary"))]
    /// let mut iter = petnames.iter(&mut rng, 4, "_");
    /// # #[cfg(all(feature = "std_rng", feature = "default_dictionary"))]
    /// println!("name: {}", iter.next().unwrap());
    /// ```
    ///
    pub fn iter<RNG>(&self, rng: &'a mut RNG, words: u8, separator: &str) -> Names<RNG>
    where
        RNG: rand::Rng,
    {
        Names {
            petnames: self,
            rng,
            words,
            separator: separator.to_string(),
        }
    }
}

#[cfg(feature = "default_dictionary")]
impl<'a> Default for Petnames<'a> {
    fn default() -> Self {
        Self::small()
    }
}

/// Iterator over a `Petnames`' word lists.
///
/// This yields the appropriate lists from which to select a word when
/// constructing a petname of `n` words. For example, if you want 3 words in
/// your petname, this will first yield the adverbs word list, then adjectives,
/// then names.
struct Lists<'a>(&'a Petnames<'a>, u8);

impl<'a> Iterator for Lists<'a> {
    type Item = &'a Words<'a>;

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.1 as usize))
    }

    fn next(&mut self) -> Option<Self::Item> {
        let Self(petnames, ref mut word) = self;
        match word {
            0 => None,
            1 => {
                *word -= 1;
                Some(&petnames.names)
            }
            2 => {
                *word -= 1;
                Some(&petnames.adjectives)
            }
            _ => {
                *word -= 1;
                Some(&petnames.adverbs)
            }
        }
    }
}

/// Iterator yielding petnames.
pub struct Names<'a, RNG>
where
    RNG: rand::Rng,
{
    petnames: &'a Petnames<'a>,
    rng: &'a mut RNG,
    words: u8,
    separator: String,
}

impl<'a, RNG> Names<'a, RNG>
where
    RNG: rand::Rng,
{
    /// Calculate the cardinality of this iterator; see `Petnames::cardinality`.
    #[allow(dead_code)]
    pub fn cardinality(&self) -> u128 {
        self.petnames.cardinality(self.words)
    }
}

impl<'a, RNG> Iterator for Names<'a, RNG>
where
    RNG: rand::Rng,
{
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        Some(
            self.petnames
                .generate(self.rng, self.words, &self.separator),
        )
    }
}
