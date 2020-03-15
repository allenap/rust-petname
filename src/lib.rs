use rand::seq::SliceRandom;

/// Convenience function to generate a new petname from default word lists.
#[allow(dead_code)]
pub fn petname(words: u8, separator: &str) -> String {
    Petnames::default().generate_one(words, separator)
}

/// Word lists and the logic to combine them into _petnames_.
///
/// A _petname_ with `n` words will contain, in order:
///
///   * 1 adjective when `n >= 2`, otherwise 0 adjectives.
///   * `n - 2` adverbs when `n >= 2`.
///   * 1 name / noun.
///
pub struct Petnames<'a> {
    pub adjectives: Vec<&'a str>,
    pub adverbs: Vec<&'a str>,
    pub names: Vec<&'a str>,
}

impl<'a> Petnames<'a> {
    /// Constructs a new `Petnames` from the default (small) word lists.
    pub fn default() -> Self {
        Self::small()
    }

    /// Constructs a new `Petnames` from the small word lists.
    pub fn small() -> Self {
        Self::init(
            include_str!("../words/small/adjectives.txt"),
            include_str!("../words/small/adverbs.txt"),
            include_str!("../words/small/names.txt"),
        )
    }

    /// Constructs a new `Petnames` from the medium word lists.
    pub fn medium() -> Self {
        Self::init(
            include_str!("../words/medium/adjectives.txt"),
            include_str!("../words/medium/adverbs.txt"),
            include_str!("../words/medium/names.txt"),
        )
    }

    /// Constructs a new `Petnames` from the large word lists.
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
    /// let mut petnames = petname::Petnames::default();
    /// petnames.retain(|s| s.starts_with("b"));
    /// petnames.generate_one(2, ".");
    /// ```
    ///
    /// This is merely a convenience wrapper that applies the same predicate to
    /// the adjectives, adverbs, and names lists.
    ///
    pub fn retain<F>(&mut self, predicate: F)
    where
        F: Fn(&&str) -> bool,
    {
        self.adjectives.retain(&predicate);
        self.adverbs.retain(&predicate);
        self.names.retain(&predicate);
    }

    /// Calculate the cardinality of this `Petnames`.
    ///
    /// If this is low, names may be repeated by the generator with a higher
    /// frequency than your use-case may allow. If it is 0 (zero) the generator
    /// will panic (unless `words` is also zero).
    ///
    /// This can saturate. If the total possible combinations of words exceeds
    /// `u128::MAX` then this will return `u128::MAX`.
    #[allow(dead_code)]
    pub fn cardinality(&self, words: u8) -> u128 {
        let mut total: u128 = if words == 0 { 0 } else { 1 };
        for num in (0..words).rev() {
            total = total.saturating_mul(match num {
                0 => self.names.len() as u128,
                1 => self.adjectives.len() as u128,
                _ => self.adverbs.len() as u128,
            });
        }
        total
    }

    /// Generate a new petname.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let mut rng = rand::thread_rng();
    /// petname::Petnames::default().generate(&mut rng, 7, ":");
    /// ```
    ///
    /// # Panics
    ///
    /// If a word list is empty.
    ///
    pub fn generate<RNG>(&self, rng: &mut RNG, words: u8, separator: &str) -> String
    where
        RNG: rand::Rng,
    {
        // Adverbs all the way, finishing with adjective then name.
        let mut parts = Vec::with_capacity(words as usize);
        for num in (0..words).rev() {
            parts.push(*match num {
                0 => self.names.choose(rng).unwrap(),
                1 => self.adjectives.choose(rng).unwrap(),
                _ => self.adverbs.choose(rng).unwrap(),
            });
        }
        parts.join(separator)
    }

    /// Generate a single new petname.
    ///
    /// This is like `generate` but uses `rand::thread_rng` as the random
    /// source. For efficiency use `generate` when creating multiple names, or
    /// when you want to use a custom source of randomness.
    pub fn generate_one(&self, words: u8, separator: &str) -> String {
        self.generate(&mut rand::thread_rng(), words, separator)
    }
}

impl<'a> Default for Petnames<'a> {
    fn default() -> Self {
        Self::default()
    }
}

#[cfg(test)]
mod tests {

    use super::{petname, Petnames};
    use rand;

    #[test]
    fn default_petnames_has_adjectives() {
        let petnames = Petnames::default();
        assert_ne!(petnames.adjectives.len(), 0);
    }

    #[test]
    fn default_petnames_has_adverbs() {
        let petnames = Petnames::default();
        assert_ne!(petnames.adverbs.len(), 0);
    }

    #[test]
    fn default_petnames_has_names() {
        let petnames = Petnames::default();
        assert_ne!(petnames.names.len(), 0);
    }

    #[test]
    fn default_petnames_has_non_zero_cardinality() {
        let petnames = Petnames::default();
        // This test will need to be adjusted when word lists change.
        assert_eq!(0, petnames.cardinality(0));
        assert_eq!(456, petnames.cardinality(1));
        assert_eq!(204744, petnames.cardinality(2));
        assert_eq!(53438184, petnames.cardinality(3));
        assert_eq!(13947366024, petnames.cardinality(4));
    }

    #[test]
    fn generate_uses_adverb_adjective_name() {
        let petnames = Petnames {
            adjectives: vec!["adjective"],
            adverbs: vec!["adverb"],
            names: vec!["name"],
        };
        assert_eq!(
            petnames.generate(&mut rand::thread_rng(), 3, "-"),
            "adverb-adjective-name"
        );
    }

    #[test]
    fn petname_renders_desired_number_of_words() {
        assert_eq!(petname(7, "-").split("-").count(), 7);
    }

    #[test]
    fn petname_renders_with_desired_separator() {
        assert_eq!(petname(7, "@").split("@").count(), 7);
    }
}
