//! English petname generator.

use alloc::{borrow::Cow, string::String};

use rand::seq::IndexedRandom;

use crate::{Generator, List, Lists, Namer, Words};

/// Word lists and the logic to combine them into English _petnames_.
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
    /// Constructs a new [`Petnames`] from the small word lists.
    ///
    /// These come from the upstream [petname][] project.
    ///
    /// [petname]: https://github.com/dustinkirkland/petname
    #[cfg(feature = "default-words")]
    pub fn small() -> Self {
        crate::english!("words/small")
    }

    /// Constructs a new [`Petnames`] from the medium word lists.
    ///
    /// These come from the upstream [petname][] project.
    ///
    /// [petname]: https://github.com/dustinkirkland/petname
    #[cfg(feature = "default-words")]
    pub fn medium() -> Self {
        crate::english!("words/medium")
    }

    /// Constructs a new [`Petnames`] from the large word lists.
    ///
    /// These come from the upstream [petname][] project.
    ///
    /// [petname]: https://github.com/dustinkirkland/petname
    #[cfg(feature = "default-words")]
    pub fn large() -> Self {
        crate::english!("words/large")
    }

    /// Constructs a new [`Petnames`] from the given word lists.
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
    /// # use petname::lang::english::Petnames;
    /// # #[cfg(feature = "default-words")] {
    /// let mut petnames = Petnames::default();
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

    /// Calculate the cardinality of this [`Petnames`].
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
    /// # use petname::lang::english::Petnames;
    /// # #[cfg(all(feature = "default-rng", feature = "default-words"))]
    /// let name = Petnames::default()
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
