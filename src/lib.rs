use rand::seq::SliceRandom;

/// Convenience function to generate a new petname from default word lists.
pub fn petname(words: u8, separator: &str) -> String {
    Petnames::default().generate(&mut rand::thread_rng(), words, separator)
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
    pub fn default() -> Petnames<'a> {
        Self::small()
    }

    /// Constructs a new `Petnames` from the small word lists.
    pub fn small() -> Petnames<'a> {
        Self::init(
            include_str!("../words/small/adjectives.txt"),
            include_str!("../words/small/adverbs.txt"),
            include_str!("../words/small/names.txt"),
        )
    }

    /// Constructs a new `Petnames` from the medium word lists.
    pub fn medium() -> Petnames<'a> {
        Self::init(
            include_str!("../words/medium/adjectives.txt"),
            include_str!("../words/medium/adverbs.txt"),
            include_str!("../words/medium/names.txt"),
        )
    }

    /// Constructs a new `Petnames` from the large word lists.
    pub fn large() -> Petnames<'a> {
        Self::init(
            include_str!("../words/large/adjectives.txt"),
            include_str!("../words/large/adverbs.txt"),
            include_str!("../words/large/names.txt"),
        )
    }

    fn init(adjectives: &'a str, adverbs: &'a str, names: &'a str) -> Petnames<'a> {
        Self {
            adjectives: adjectives.split_whitespace().collect(),
            adverbs: adverbs.split_whitespace().collect(),
            names: names.split_whitespace().collect(),
        }
    }

    /// Generate a new petname.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let mut rng = rand::thread_rng();
    /// petname::Petnames::default().generate(&mut rng, 7, ":");
    /// ```
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
}

impl<'a> Default for Petnames<'a> {
    fn default() -> Self { Self::default() }
}

#[cfg(test)]
mod tests {

    use rand;
    use super::{petname, Petnames};

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
