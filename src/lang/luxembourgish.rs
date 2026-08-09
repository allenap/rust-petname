//! Luxembourgish petname generator.
//!
//! Luxembourgish (Lëtzebuergesch) is the first target to need grammatical
//! gender and join-time sandhi, which is why the [`Generator`] trait hands the
//! model the whole buffer.
//!
//! Two features set it apart from English and Turkish:
//!
//!   * **Gender agreement.** Every [`Noun`] is masculine, feminine, or neuter,
//!     and an attributive adjective agrees with it in the articleless strong
//!     nominative: masculine takes `-en` (_groussen Hond_), feminine is bare
//!     (_grouss Kaz_), neuter takes `-t` (_grousst Haus_). Irregular forms are
//!     common – _héich_ → _héijen_ (ch → j), _gutt_ → _gudden_, _rout_ →
//!     _rouden_ (t → d) – so each [`Adjective`] carries all three forms
//!     explicitly rather than deriving them.
//!
//!   * **The Eifeler Regel.** A final `-n` – or `-nn` – drops before a word that
//!     begins with anything other than `n, d, t, z, h` or a vowel. It is applied
//!     at the join between words: _groussen_ + _mupp_ → `grousse-mupp`, but
//!     _groussen_ + _hond_ → `groussen-hond` and _groussen_ + _apel_ →
//!     `groussen-apel`. The separator plays no part; the rule is about the
//!     sounds either side of the join. The final word of a name is
//!     phrase-final, so its `-n` (if any) is kept.

use alloc::borrow::Cow;
use alloc::string::String;

use rand::seq::IndexedRandom;

use crate::{Generator, List, Lists, Namer};

/// Grammatical gender of a [`Noun`], which an attributive [`Adjective`] agrees
/// with.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Gender {
    Masculine,
    Feminine,
    Neuter,
}

/// A noun together with its grammatical gender.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Noun<'a> {
    pub word: &'a str,
    pub gender: Gender,
}

impl<'a> Noun<'a> {
    /// A noun with the given gender.
    pub const fn new(word: &'a str, gender: Gender) -> Self {
        Self { word, gender }
    }
}

/// An attributive adjective, carrying its three gender-inflected forms.
///
/// e.g. `Adjective { masculine: "groussen", feminine: "grouss", neuter: "grousst" }`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Adjective<'a> {
    pub masculine: &'a str,
    pub feminine: &'a str,
    pub neuter: &'a str,
}

impl<'a> Adjective<'a> {
    /// An adjective from its masculine, feminine, and neuter forms.
    pub const fn new(masculine: &'a str, feminine: &'a str, neuter: &'a str) -> Self {
        Self { masculine, feminine, neuter }
    }

    /// The form that agrees with the given gender.
    pub const fn agree(&self, gender: Gender) -> &'a str {
        match gender {
            Gender::Masculine => self.masculine,
            Gender::Feminine => self.feminine,
            Gender::Neuter => self.neuter,
        }
    }
}

/// Word lists and the logic to combine them into Luxembourgish _petnames_.
///
/// A petname with `n` words contains, in order:
///
///   * `n - 2` intensifier adverbs when `n >= 2`, otherwise 0.
///   * 1 adjective, agreeing with the noun's gender, when `n >= 2`, otherwise 0.
///   * 1 noun when `n >= 1`, otherwise 0.
///
/// Adjacent words are joined with the separator, with the Eifeler Regel applied
/// across each join (see the [module docs][self]).
///
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Petnames<'a> {
    pub adjectives: Cow<'a, [Adjective<'a>]>,
    /// Intensifiers such as `ganz` ("very") and `richteg` ("really").
    pub adverbs: Cow<'a, [&'a str]>,
    pub nouns: Cow<'a, [Noun<'a>]>,
}

impl<'a> Petnames<'a> {
    /// Constructs a new Luxembourgish generator from the built-in word lists.
    #[cfg(feature = "default-words")]
    pub fn small() -> Self {
        crate::luxembourgish!("words/luxembourgish")
    }

    /// Keep words matching a predicate.
    ///
    /// This is a convenience wrapper that applies the same predicate to the
    /// adjectives, adverbs, and nouns (by their word) lists. An adjective is
    /// kept only when *all three* of its gender forms match, since any one of
    /// them can be the form that lands in a name; that's what makes a limit on
    /// word length, say, hold for the generated name and not merely for the
    /// lemma.
    pub fn retain<F>(&mut self, mut predicate: F)
    where
        F: FnMut(&str) -> bool,
    {
        self.adjectives.to_mut().retain(|adjective| {
            predicate(adjective.masculine) && predicate(adjective.feminine) && predicate(adjective.neuter)
        });
        self.adverbs.to_mut().retain(|word| predicate(word));
        self.nouns.to_mut().retain(|noun| predicate(noun.word));
    }

    /// Calculate the cardinality of this generator.
    ///
    /// This can saturate. If the total possible combinations of words exceeds
    /// `u128::MAX` then this will return `u128::MAX`. Gender agreement and the
    /// Eifeler Regel are deterministic, so they do not affect the count.
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

impl Generator for Petnames<'_> {
    fn generate_into(&self, buf: &mut String, rng: &mut dyn rand::Rng, words: u8, separator: &str) {
        // Choose the noun first: it fixes the gender that the adjective agrees
        // with, even though the noun itself is emitted last. With no nouns to
        // choose from, fall back to the feminine – the bare – form.
        let noun = self.nouns.choose(rng);
        let gender = noun.map_or(Gender::Feminine, |noun| noun.gender);

        // Write each word one step behind the cursor: the Eifeler Regel needs to
        // see what follows a word before that word can be written out. Slots
        // whose list is empty are skipped, matching the "fewer words than
        // requested" behaviour of the other generators.
        let mut pending: Option<&str> = None;
        for list in Lists::new(words) {
            let word = match list {
                List::Adverb => self.adverbs.choose(rng).copied(),
                List::Adjective => self.adjectives.choose(rng).map(|adjective| adjective.agree(gender)),
                List::Noun => noun.map(|noun| noun.word),
            };
            if let Some(word) = word {
                if let Some(previous) = pending.replace(word) {
                    buf.push_str(elide_n(previous, word));
                    buf.push_str(separator);
                }
            }
        }

        // The last word is phrase-final, so it keeps its `-n`.
        if let Some(last) = pending {
            buf.push_str(last);
        }
    }
}

/// Apply the Eifeler Regel to `word`, given the word that follows it: a final
/// `-n` or `-nn` is dropped when the next word begins with a sound that is
/// neither a vowel nor one of `n, d, t, z, h`. Any other `word` is returned
/// unchanged.
fn elide_n<'a>(word: &'a str, next: &str) -> &'a str {
    match next.chars().next() {
        Some(next) if triggers_elision(next) => {
            // Both `-n` and `-nn` go, not just the last `n`: _dënn_ + _mupp_ is
            // `dë-mupp`, never `dën-mupp`.
            word.strip_suffix("nn").or_else(|| word.strip_suffix('n')).unwrap_or(word)
        }
        _ => word,
    }
}

/// Whether a following sound triggers `-n` elision: anything that is not a vowel
/// and not one of `n, d, t, z, h`.
fn triggers_elision(next: char) -> bool {
    // Fold case once, so that only the lower-case forms need listing below.
    let next = next.to_lowercase().next().unwrap_or(next);
    let keeps_n = matches!(next, 'n' | 'd' | 't' | 'z' | 'h');
    let is_vowel = matches!(
        next,
        'a' | 'e'
            | 'i'
            | 'o'
            | 'u'
            | 'é'
            | 'è'
            | 'ë'
            | 'ä'
            | 'ö'
            | 'ü'
            | 'á'
            | 'à'
            | 'â'
            | 'ê'
            | 'î'
            | 'ô'
            | 'û'
    );
    !(keeps_n || is_vowel)
}

#[cfg(test)]
mod tests {
    use alloc::borrow::Cow;
    use alloc::vec;

    use super::{elide_n, Adjective, Gender, Noun, Petnames};

    fn sample() -> Petnames<'static> {
        Petnames {
            adjectives: Cow::Owned(vec![
                Adjective::new("groussen", "grouss", "grousst"),
                Adjective::new("klengen", "kleng", "klengt"),
            ]),
            adverbs: Cow::Owned(vec!["ganz", "richteg"]),
            nouns: Cow::Owned(vec![
                Noun::new("mupp", Gender::Masculine),
                Noun::new("kaz", Gender::Feminine),
                Noun::new("haus", Gender::Neuter),
            ]),
        }
    }

    #[test]
    fn agree_selects_form_by_gender() {
        let adjective = Adjective::new("groussen", "grouss", "grousst");
        assert_eq!(adjective.agree(Gender::Masculine), "groussen");
        assert_eq!(adjective.agree(Gender::Feminine), "grouss");
        assert_eq!(adjective.agree(Gender::Neuter), "grousst");
    }

    #[test]
    fn eifeler_regel_elides_n_only_before_triggering_sounds() {
        // Dropped before consonants outside the keep-set.
        assert_eq!(elide_n("groussen", "mupp"), "grousse");
        assert_eq!(elide_n("groussen", "bam"), "grousse");
        assert_eq!(elide_n("groussen", "Kaz"), "grousse"); // case-insensitive
                                                           // Kept before n, d, t, z, h …
        assert_eq!(elide_n("groussen", "hond"), "groussen");
        assert_eq!(elide_n("groussen", "donner"), "groussen");
        assert_eq!(elide_n("groussen", "zocker"), "groussen");
        // … and before a vowel, accented or not.
        assert_eq!(elide_n("groussen", "apel"), "groussen");
        assert_eq!(elide_n("groussen", "éiweg"), "groussen");
        // A word not ending in `n` is left alone …
        assert_eq!(elide_n("grouss", "mupp"), "grouss");
        // … and `-nn` goes whole, rather than being left half-stripped.
        assert_eq!(elide_n("dënn", "mupp"), "dë");
        assert_eq!(elide_n("dënn", "hond"), "dënn");
    }

    // Generation needs a seedable RNG, which `StdRng` provides only when
    // `default-rng` is enabled.
    #[cfg(feature = "default-rng")]
    fn generate(lb: &Petnames, words: u8, seed: u64) -> alloc::vec::Vec<alloc::string::String> {
        use rand::SeedableRng;
        let namer = lb.namer(words, "-");
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
        let lb = sample();
        for words in 1..=5u8 {
            for name in generate(&lb, words, 1) {
                assert_eq!(name.split('-').count(), words as usize, "name was {name:?}");
            }
        }
    }

    #[cfg(feature = "default-rng")]
    #[test]
    fn agreement_and_elision_are_applied() {
        use rand::SeedableRng;
        // Single-entry lists make the output deterministic regardless of the rng.
        let masculine = Petnames {
            adjectives: Cow::Owned(vec![Adjective::new("groussen", "grouss", "grousst")]),
            adverbs: Cow::Owned(vec![]),
            nouns: Cow::Owned(vec![Noun::new("mupp", Gender::Masculine)]),
        };
        let feminine =
            Petnames { nouns: Cow::Owned(vec![Noun::new("kaz", Gender::Feminine)]), ..masculine.clone() };
        let kept =
            Petnames { nouns: Cow::Owned(vec![Noun::new("hond", Gender::Masculine)]), ..masculine.clone() };
        let mut rng = rand::rngs::StdRng::seed_from_u64(1);
        let render = |lb: &Petnames, rng: &mut rand::rngs::StdRng| {
            let mut buf = alloc::string::String::new();
            lb.namer(2, "-").generate_into(&mut buf, rng);
            buf
        };
        // Masculine "groussen" elides its -n before "mupp".
        assert_eq!(render(&masculine, &mut rng), "grousse-mupp");
        // Feminine form is bare, so nothing to elide.
        assert_eq!(render(&feminine, &mut rng), "grouss-kaz");
        // Masculine -n is kept before "hond" (h is in the keep-set).
        assert_eq!(render(&kept, &mut rng), "groussen-hond");
    }

    #[cfg(feature = "default-rng")]
    #[test]
    fn deterministic_under_seed() {
        let lb = sample();
        assert_eq!(generate(&lb, 3, 42), generate(&lb, 3, 42));
    }

    #[test]
    fn cardinality_counts_combinations() {
        let lb = sample(); // 2 adjectives, 2 adverbs, 3 nouns.
        assert_eq!(lb.cardinality(1), 3); // noun
        assert_eq!(lb.cardinality(2), 6); // adjective * noun
        assert_eq!(lb.cardinality(3), 12); // adverb * adjective * noun
        assert_eq!(lb.cardinality(0), 0);
    }

    #[cfg(feature = "default-words")]
    #[test]
    fn small_parses_genders_and_three_forms() {
        let lb = Petnames::small();
        // A known adjective keeps all three forms…
        assert!(lb.adjectives.contains(&Adjective::new("groussen", "grouss", "grousst")));
        // …and a known noun keeps its gender.
        assert!(lb.nouns.contains(&Noun::new("kaz", Gender::Feminine)));
        // Comment/header words must not leak in as data.
        assert!(!lb.nouns.iter().any(|noun| noun.word.starts_with('#')));
        assert!(!lb.adjectives.iter().any(|adjective| adjective.feminine.contains(':')));
    }

    #[test]
    fn retain_filters_all_lists() {
        let mut lb = sample();
        lb.retain(|word| word.chars().count() <= 4);
        // adjectives: no form of either is short enough.
        assert!(lb.adjectives.is_empty());
        // adverbs: "ganz" (4) kept, "richteg" (7) dropped.
        assert_eq!(lb.adverbs.len(), 1);
        // nouns by word: "mupp" (4), "kaz" (3), "haus" (4) all kept.
        assert_eq!(lb.nouns.len(), 3);
    }

    #[test]
    fn retain_judges_adjectives_on_every_gender_form() {
        // "grouss" and "kleng" are both within 6 characters, but their longer
        // masculine forms – "groussen", "klengen" – are the ones that would
        // appear with a masculine noun, so neither adjective survives.
        let mut lb = sample();
        lb.retain(|word| word.chars().count() <= 6);
        assert!(lb.adjectives.is_empty());
        // Relax the limit enough for every form and both come back.
        let mut lb = sample();
        lb.retain(|word| word.chars().count() <= 8);
        assert_eq!(lb.adjectives.len(), 2);
    }

    /// The built-in adjectives follow a pattern regular enough to police, which
    /// is worth doing because the lists are meant to be extended by people who
    /// won't be running the rest of the test suite in their heads: the masculine
    /// ends in `-n` (the Eifeler Regel has nothing to act on otherwise), and the
    /// neuter is the feminine plus `-t`, except after a stem-final `-t`/`-d`
    /// where nothing is added. Genuine exceptions go in `IRREGULAR_NEUTER`.
    #[cfg(feature = "default-words")]
    #[test]
    fn small_adjectives_are_regularly_formed() {
        /// Feminine forms whose neuter does not follow the rule above.
        const IRREGULAR_NEUTER: &[&str] = &[];

        for adjective in Petnames::small().adjectives.iter() {
            assert!(
                adjective.masculine.ends_with('n'),
                "masculine {:?} should end in -n",
                adjective.masculine
            );
            if IRREGULAR_NEUTER.contains(&adjective.feminine) {
                continue;
            }
            let expected = if adjective.feminine.ends_with(['t', 'd']) {
                alloc::string::String::from(adjective.feminine)
            } else {
                alloc::format!("{}t", adjective.feminine)
            };
            assert_eq!(
                adjective.neuter, expected,
                "neuter of {:?} should be {expected:?}",
                adjective.feminine
            );
        }
    }
}
