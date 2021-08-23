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
//! # #[cfg(feature = "default-rng")]
//! let mut rng = rand::thread_rng();
//! # #[cfg(all(feature = "default-rng", feature = "default-words"))]
//! let pname = petname::Petnames::default().generate(&mut rng, 7, ":");
//! ```
//!
//! It may be more convenient to use the default random number generator:
//!
//! ```rust
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
    string::{String, ToString},
    vec::Vec,
};

use itertools::Itertools;
use rand::seq::SliceRandom;

/// Convenience function to generate a new petname from default word lists.
#[allow(dead_code)]
#[cfg(feature = "default-rng")]
#[cfg(feature = "default-words")]
pub fn petname(words: u8, separator: &str) -> String {
    Petnames::default().generate_one(words, separator)
}

/// A word list.
pub type Words<'a> = Cow<'a, [&'a str]>;

/// Word lists and the logic to combine them into _petnames_.
///
/// A _petname_ with `n` words will contain, in order:
///
///   * `n - 2` adverbs when `n >= 2`, otherwise 0 adverbs.
///   * 1 adjective when `n >= 2`, otherwise 0 adjectives.
///   * 1 name / noun when `n >= 1`, otherwise 0 names.
///
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Petnames<'a> {
    pub adjectives: Words<'a>,
    pub adverbs: Words<'a>,
    pub names: Words<'a>,
}

#[cfg(feature = "default-words")]
mod words {
    include!(concat!(env!("OUT_DIR"), "/words.rs"));
}

impl<'a> Petnames<'a> {
    /// Constructs a new `Petnames` from the small word lists.
    #[cfg(feature = "default-words")]
    pub fn small() -> Self {
        Self {
            adjectives: Cow::from(&words::small::ADJECTIVES[..]),
            adverbs: Cow::from(&words::small::ADVERBS[..]),
            names: Cow::from(&words::small::NAMES[..]),
        }
    }

    /// Constructs a new `Petnames` from the medium word lists.
    #[cfg(feature = "default-words")]
    pub fn medium() -> Self {
        Self {
            adjectives: Cow::from(&words::medium::ADJECTIVES[..]),
            adverbs: Cow::from(&words::medium::ADVERBS[..]),
            names: Cow::from(&words::medium::NAMES[..]),
        }
    }

    /// Constructs a new `Petnames` from the large word lists.
    #[cfg(feature = "default-words")]
    pub fn large() -> Self {
        Self {
            adjectives: Cow::from(&words::large::ADJECTIVES[..]),
            adverbs: Cow::from(&words::large::ADVERBS[..]),
            names: Cow::from(&words::large::NAMES[..]),
        }
    }

    /// Constructs a new `Petnames` from the given word lists.
    ///
    /// The words are extracted from the given strings by splitting on whitespace.
    pub fn new(adjectives: &'a str, adverbs: &'a str, names: &'a str) -> Self {
        Self {
            adjectives: Cow::Owned(adjectives.split_whitespace().collect()),
            adverbs: Cow::Owned(adverbs.split_whitespace().collect()),
            names: Cow::Owned(names.split_whitespace().collect()),
        }
    }

    /// Keep words matching a predicate.
    ///
    /// # Examples
    ///
    /// ```rust
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
    /// the adjectives, adverbs, and names lists.
    ///
    pub fn retain<F>(&mut self, mut predicate: F)
    where
        F: FnMut(&str) -> bool,
    {
        self.adjectives.to_mut().retain(|word| predicate(word));
        self.adverbs.to_mut().retain(|word| predicate(word));
        self.names.to_mut().retain(|word| predicate(word));
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
        Lists::new(words)
            .map(|list| match list {
                List::Adverb => self.adverbs.len() as u128,
                List::Adjective => self.adjectives.len() as u128,
                List::Name => self.names.len() as u128,
            })
            .reduce(u128::saturating_mul)
            .unwrap_or(0u128)
    }

    /// Generate a new petname.
    ///
    /// # Examples
    ///
    /// ```rust
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
    pub fn generate<RNG>(&self, rng: &mut RNG, words: u8, separator: &str) -> String
    where
        RNG: rand::Rng,
    {
        Itertools::intersperse(
            Lists::new(words).filter_map(|list| match list {
                List::Adverb => self.adverbs.choose(rng).copied(),
                List::Adjective => self.adjectives.choose(rng).copied(),
                List::Name => self.names.choose(rng).copied(),
            }),
            separator,
        )
        .collect::<String>()
    }

    /// Generate a single new petname.
    ///
    /// This is like `generate` but uses `rand::thread_rng` as the random
    /// source. For efficiency use `generate` when creating multiple names, or
    /// when you want to use a custom source of randomness.
    #[cfg(feature = "default-rng")]
    pub fn generate_one(&self, words: u8, separator: &str) -> String {
        self.generate(&mut rand::thread_rng(), words, separator)
    }

    /// Iterator yielding petnames.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #[cfg(all(feature = "default-rng", feature = "default-words"))]
    /// let mut rng = rand::thread_rng();
    /// # #[cfg(all(feature = "default-rng", feature = "default-words"))]
    /// let petnames = petname::Petnames::default();
    /// # #[cfg(all(feature = "default-rng", feature = "default-words"))]
    /// let mut iter = petnames.iter(&mut rng, 4, "_");
    /// # #[cfg(all(feature = "default-rng", feature = "default-words"))]
    /// println!("name: {}", iter.next().unwrap());
    /// ```
    ///
    pub fn iter<RNG>(
        &'a self,
        rng: &'a mut RNG,
        words: u8,
        separator: &str,
    ) -> impl Iterator<Item = String> + 'a
    where
        RNG: rand::Rng,
    {
        Names { petnames: self, rng, words, separator: separator.to_string() }
    }

    /// Iterator yielding unique – i.e. non-repeating – petnames.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #[cfg(all(feature = "default-rng", feature = "default-words"))]
    /// let mut rng = rand::thread_rng();
    /// # #[cfg(all(feature = "default-rng", feature = "default-words"))]
    /// let petnames = petname::Petnames::default();
    /// # #[cfg(all(feature = "default-rng", feature = "default-words"))]
    /// let mut iter = petnames.iter_non_repeating(&mut rng, 4, "_");
    /// # #[cfg(all(feature = "default-rng", feature = "default-words"))]
    /// println!("name: {}", iter.next().unwrap());
    /// ```
    ///
    pub fn iter_non_repeating<RNG>(
        &'a self,
        rng: &'a mut RNG,
        words: u8,
        separator: &str,
    ) -> impl Iterator<Item = String> + 'a
    where
        RNG: rand::Rng,
    {
        let lists: Vec<&'a Words<'a>> = Lists::new(words)
            .map(|list| match list {
                List::Adverb => &self.adverbs,
                List::Adjective => &self.adjectives,
                List::Name => &self.names,
            })
            .collect();
        NamesProduct::shuffled(&lists, rng, separator)
    }
}

#[cfg(feature = "default-words")]
impl<'a> Default for Petnames<'a> {
    /// Constructs a new `Petnames` from the default (small) word lists.
    fn default() -> Self {
        Self::small()
    }
}

/// Enum representing which word list to use.
#[derive(Debug, PartialEq)]
enum List {
    Adverb,
    Adjective,
    Name,
}

/// Iterator, yielding which word list to use next.
///
/// This yields the appropriate list – [adverbs][List::Adverb],
/// [adjectives][List::Adjective]s, [names][List::Name] –  from which to select
/// a word when constructing a petname of `n` words. For example, if you want 4
/// words in your petname, this will first yield [List::Adverb], then
/// [List::Adverb] again, then [List::Adjective], and lastly [List::Name].
#[derive(Debug, PartialEq)]
enum Lists {
    Adverb(u8),
    Adjective,
    Name,
    Done,
}

impl Lists {
    fn new(words: u8) -> Self {
        match words {
            0 => Self::Done,
            1 => Self::Name,
            2 => Self::Adjective,
            n => Self::Adverb(n - 3),
        }
    }

    fn current(&self) -> Option<List> {
        match self {
            Self::Adjective => Some(List::Adjective),
            Self::Adverb(_) => Some(List::Adverb),
            Self::Name => Some(List::Name),
            Self::Done => None,
        }
    }

    fn advance(&mut self) {
        *self = match self {
            Self::Adverb(0) => Self::Adjective,
            Self::Adverb(remaining) => Self::Adverb(*remaining - 1),
            Self::Adjective => Self::Name,
            Self::Name | Self::Done => Self::Done,
        }
    }

    fn remaining(&self) -> usize {
        match self {
            Self::Adverb(n) => (n + 3) as usize,
            Self::Adjective => 2,
            Self::Name => 1,
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
struct Names<'a, RNG>
where
    RNG: rand::Rng,
{
    petnames: &'a Petnames<'a>,
    rng: &'a mut RNG,
    words: u8,
    separator: String,
}

impl<'a, RNG> Iterator for Names<'a, RNG>
where
    RNG: rand::Rng,
{
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.petnames.generate(self.rng, self.words, &self.separator))
    }
}

/// Iterator yielding petnames from the product of given word lists.
///
/// This can be used to ensure that only unique names are produced.
struct NamesProduct<'a, ITERATOR>
where
    ITERATOR: Iterator<Item = Option<&'a str>>,
{
    iters: Vec<(ITERATOR, Option<&'a str>)>,
    separator: String,
    capacity: usize,
}

impl<'a> NamesProduct<'a, core::iter::Cycle<alloc::vec::IntoIter<Option<&'a str>>>> {
    /// Shuffles each of the given `lists` with `rng`, then cycles through the
    /// product of the lists, joining with `separator`. The leftmost list will
    /// cycle most rapidly.
    fn shuffled<RNG>(lists: &[&'a Words<'a>], rng: &'a mut RNG, separator: &str) -> Self
    where
        RNG: rand::Rng,
    {
        NamesProduct {
            iters: lists
                .iter()
                .map(|words| {
                    let mut list: Vec<Option<&'a str>> = Vec::with_capacity(words.len().saturating_add(1));
                    list.extend(words.iter().map(|word| Some(*word)));
                    list.shuffle(rng); // Could be expensive.
                    list.push(None); // Cycle marker.
                    (list.into_iter().cycle(), None)
                })
                .collect(),
            separator: separator.to_string(),
            capacity: Self::capacity(lists, separator),
        }
    }

    fn capacity(lists: &[&'a Words<'a>], separator: &str) -> usize {
        (
            // Sum of the length of the longest possible word in each word list.
            lists
                .iter()
                .filter_map(|words| words.iter().map(|word| word.len()).max())
                .fold(0usize, |sum, len| sum.saturating_add(len))
            // The total length of all separators. Careful not to wrap usize.
            + (separator.len().saturating_mul(lists.len().saturating_sub(1)))
        )
        // Things run _much_ quicker when the capacity is a power of 2. Memory
        // alignment? If so it may be enough to align at, say, 8 bytes, but this
        // works for now.
        .checked_next_power_of_two()
        // In case there are no lists, or they're all empty... or we have
        // calculated that we need more than usize::MAX capacity.
        .unwrap_or(0)
    }
}

impl<'a, ITERATOR> Iterator for NamesProduct<'a, ITERATOR>
where
    ITERATOR: Iterator<Item = Option<&'a str>>,
{
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        let mut bump = true; // Request advance of next iterator.
        for (iter, word) in self.iters.iter_mut() {
            if bump || word.is_none() {
                match iter.next() {
                    None => {
                        // This shouldn't happen because we expect the iterators
                        // to cycle. However, if it does, we're definitely done.
                        return None;
                    }
                    Some(None) => {
                        // This is the cycle end marker. We want to get another
                        // new word from this iterator, and advance the *next*
                        // iterator too.
                        match iter.next() {
                            None => return None,
                            Some(None) => return None,
                            Some(s) => *word = s,
                        }
                        bump = true
                    }
                    Some(s) => {
                        // We have a new word from this iterator, so we do not
                        // yet need to advance the next iterator.
                        *word = s;
                        bump = false
                    }
                }
            }
        }
        if bump {
            // We reached the end of the last iterator, hence we're done.
            None
        } else {
            // We may be able to construct a word!
            self.iters.iter().try_fold(String::with_capacity(self.capacity), |acc, (_, w)| match (acc, *w) {
                (s, Some(w)) if s.is_empty() => Some(s + w),
                (s, Some(w)) => Some(s + &self.separator + w),
                _ => None,
            })
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
        assert_eq!(super::Lists::Name, lists);
        assert_eq!(Some(super::List::Name), lists.next());
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
