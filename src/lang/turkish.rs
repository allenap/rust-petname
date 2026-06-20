//! Turkish petname generator.
//!
//! After English, Turkish is grammatically the simplest target for petnames: it
//! has no grammatical gender, attributive adjectives do not agree with the
//! noun, and adjectives precede the noun as in English. So `adjective + noun`
//! juxtaposition is already correct Turkish – `kırmızı kedi` ("red cat") – with
//! no linking suffix.
//!
//! Its one distinctive flourish is partial reduplication for intensification
//! (_pekiştirme_): `kırmızı` → `kıpkırmızı` ("bright red"), `beyaz` →
//! `bembeyaz` ("pure white"). The inserted consonant is lexicalised rather than
//! rule-derivable, so the emphatic form is carried as data on each
//! [`Adjective`] that has one. A reduplicated adjective is a single token, so
//! it is used only when the petname has no separate intensifier adverbs (i.e. a
//! two-word name); otherwise the base form is used to avoid doubling up the
//! intensification (`çok-kırmızı-kedi`, not `çok-kıpkırmızı-kedi`).

use alloc::borrow::Cow;
use alloc::string::String;

use rand::{seq::IndexedRandom, RngExt};

use crate::{Generator, List, Lists, Namer};

/// An attributive adjective, with an optional emphatic (reduplicated) form.
///
/// The emphatic form, when present, already carries the sense of "very" or
/// "intensely", e.g. `Adjective { word: "kırmızı", emphatic: Some("kıpkırmızı") }`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Adjective<'a> {
    pub word: &'a str,
    pub emphatic: Option<&'a str>,
}

impl<'a> Adjective<'a> {
    /// An adjective with no emphatic form.
    pub const fn plain(word: &'a str) -> Self {
        Self { word, emphatic: None }
    }

    /// An adjective with an emphatic (reduplicated) form.
    pub const fn emphatic(word: &'a str, emphatic: &'a str) -> Self {
        Self { word, emphatic: Some(emphatic) }
    }
}

/// Word lists and the logic to combine them into Turkish petnames.
///
/// A petname with `n` words contains, in order:
///
///   * `n - 2` intensifier adverbs when `n >= 2`, otherwise 0.
///   * 1 adjective when `n >= 2`, otherwise 0.
///   * 1 noun when `n >= 1`, otherwise 0.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Turkish<'a> {
    pub adjectives: Cow<'a, [Adjective<'a>]>,
    /// Intensifiers such as `çok` ("very") and `oldukça` ("quite").
    pub adverbs: Cow<'a, [&'a str]>,
    pub nouns: Cow<'a, [&'a str]>,
}

impl<'a> Turkish<'a> {
    /// Constructs a new Turkish generator from the built-in word lists.
    #[cfg(feature = "default-words")]
    pub fn small() -> Self {
        crate::turkish!("words/turkish")
    }

    /// Keep words matching a predicate.
    ///
    /// This is a convenience wrapper that applies the same predicate to the
    /// adjectives (by their base form), adverbs, and nouns lists.
    pub fn retain<F>(&mut self, mut predicate: F)
    where
        F: FnMut(&str) -> bool,
    {
        self.adjectives.to_mut().retain(|adjective| predicate(adjective.word));
        self.adverbs.to_mut().retain(|word| predicate(word));
        self.nouns.to_mut().retain(|word| predicate(word));
    }

    /// Calculate the cardinality of this generator.
    ///
    /// This can saturate. If the total possible combinations of words exceeds
    /// `u128::MAX` then this will return `u128::MAX`. The emphatic adjective
    /// forms are not counted as distinct combinations.
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
    pub fn namer<'b>(&'b self, words: u8, separator: &'b str) -> Namer<'b, Self> {
        Namer { generator: self, words, separator }
    }
}

impl Generator for Turkish<'_> {
    fn generate_into(&self, buf: &mut String, rng: &mut dyn rand::Rng, words: u8, separator: &str) {
        // Emphatic (reduplicated) adjectives are themselves a token meaning
        // "very X", so only reach for them when there are no separate adverb
        // intensifiers, i.e. a two-word name.
        let allow_emphatic = words == 2;
        for list in Lists::new(words) {
            match list {
                List::Adverb => {
                    if let Some(word) = self.adverbs.choose(rng).copied() {
                        buf.push_str(word);
                        buf.push_str(separator);
                    }
                }
                List::Adjective => {
                    if let Some(adjective) = self.adjectives.choose(rng) {
                        // Use the emphatic form only for two-word names, and
                        // then only half the time.
                        let word = match adjective.emphatic {
                            Some(form) if allow_emphatic && rng.random_bool(0.5) => form,
                            _ => adjective.word,
                        };
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

#[cfg(test)]
mod tests {
    use alloc::borrow::Cow;
    use alloc::vec;

    use super::{Adjective, Turkish};

    fn sample() -> Turkish<'static> {
        Turkish {
            adjectives: Cow::Owned(vec![
                Adjective::emphatic("kırmızı", "kıpkırmızı"),
                Adjective::plain("güzel"),
            ]),
            adverbs: Cow::Owned(vec!["çok", "oldukça"]),
            nouns: Cow::Owned(vec!["kedi", "köpek"]),
        }
    }

    // Generation needs a seedable RNG, which `StdRng` provides only when
    // `default-rng` is enabled.
    #[cfg(feature = "default-rng")]
    fn generate(turkish: &Turkish, words: u8, seed: u64) -> alloc::vec::Vec<alloc::string::String> {
        use rand::SeedableRng;
        let namer = turkish.namer(words, "-");
        let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
        (0..50)
            .map(|_| {
                let mut buf = alloc::string::String::new();
                namer.generate_into(&mut buf, &mut rng);
                buf
            })
            .collect()
    }

    #[cfg(feature = "default-rng")]
    #[test]
    fn token_count_matches_words() {
        let turkish = sample();
        for words in 1..=5u8 {
            for name in generate(&turkish, words, 1) {
                assert_eq!(name.split('-').count(), words as usize, "name was {name:?}");
            }
        }
    }

    #[cfg(feature = "default-rng")]
    #[test]
    fn emphatic_form_only_when_two_words() {
        let turkish = sample();
        // With three words there is an adverb, so the emphatic form must never
        // appear.
        assert!(generate(&turkish, 3, 7).iter().all(|name| !name.contains("kıpkırmızı")));
        // With two words the emphatic form should appear at least sometimes.
        assert!(generate(&turkish, 2, 7).iter().any(|name| name.contains("kıpkırmızı")));
    }

    #[cfg(feature = "default-rng")]
    #[test]
    fn deterministic_under_seed() {
        let turkish = sample();
        assert_eq!(generate(&turkish, 3, 42), generate(&turkish, 3, 42));
    }

    #[test]
    fn cardinality_counts_combinations() {
        let turkish = sample(); // 2 adjectives, 2 adverbs, 2 nouns.
        assert_eq!(turkish.cardinality(1), 2); // noun
        assert_eq!(turkish.cardinality(2), 4); // adjective * noun
        assert_eq!(turkish.cardinality(3), 8); // adverb * adjective * noun
        assert_eq!(turkish.cardinality(0), 0);
    }

    #[cfg(feature = "default-words")]
    #[test]
    fn small_parses_emphatic_and_strips_comments() {
        let turkish = Turkish::small();
        // An annotated adjective keeps its emphatic form...
        assert!(turkish.adjectives.contains(&Adjective::emphatic("kırmızı", "kıpkırmızı")));
        // ...and a plain one has none.
        assert!(turkish.adjectives.contains(&Adjective::plain("güzel")));
        assert!(turkish.nouns.contains(&"kedi"));
        // Comment words must not leak in as data.
        assert!(!turkish.nouns.iter().any(|word| word.starts_with('#')));
        assert!(!turkish.nouns.contains(&"Animals"));
        assert!(!turkish.adjectives.iter().any(|adjective| adjective.word.contains('=')));
    }

    #[test]
    fn retain_filters_all_lists() {
        let mut turkish = sample();
        turkish.retain(|word| word.chars().count() <= 5);
        // "kırmızı" (7), "oldukça" (7) and "köpek" (5) -> "oldukça" dropped.
        assert!(turkish.adjectives.iter().all(|adjective| adjective.word.chars().count() <= 5));
        assert!(turkish.adverbs.iter().all(|word| word.chars().count() <= 5));
        assert!(turkish.nouns.iter().all(|word| word.chars().count() <= 5));
        assert_eq!(turkish.adverbs.len(), 1); // only "çok" remains.
    }
}
