extern crate rand;


/// Convenience function to generate a new petname from default word lists.
pub fn petname(words: u8, separator: &str) -> String {
    Petnames::default().generate(
        &mut rand::thread_rng(), words, separator)
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

    /// Constructs a new `Petnames` with default word lists.
    pub fn default() -> Petnames<'a> {
        let adjectives = concat!(
            include_str!("../words/large/adjectives.txt"), "\n",
            include_str!("../words/medium/adjectives.txt"), "\n",
            include_str!("../words/small/adjectives.txt"), "\n",
        );
        let adverbs = concat!(
            include_str!("../words/large/adverbs.txt"), "\n",
            include_str!("../words/medium/adverbs.txt"), "\n",
            include_str!("../words/small/adverbs.txt"), "\n",
        );
        let names = concat!(
            include_str!("../words/large/names.txt"), "\n",
            include_str!("../words/medium/names.txt"), "\n",
            include_str!("../words/small/names.txt"), "\n",
        );
        Self{
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
    /// # extern crate rand;
    /// # extern crate petname;
    /// let mut rng = rand::thread_rng();
    /// petname::Petnames::default().generate(&mut rng, 7, ":");
    /// ```
    pub fn generate<RNG>(
        &self, rng: &mut RNG, words: u8, separator: &str) -> String
        where RNG: rand::Rng
    {
        // Adverbs all the way, finishing with adjective then name.
        let mut parts = Vec::with_capacity(words as usize);
        for num in (0..words).rev() {
            parts.push(*match num {
                0 => rng.choose(&self.names).unwrap(),
                1 => rng.choose(&self.adjectives).unwrap(),
                _ => rng.choose(&self.adverbs).unwrap(),
            });
        };
        parts.join(separator)
    }

}


#[cfg(test)]
mod tests {

    use super::{petname, Petnames, rand};

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
        let petnames = Petnames{
            adjectives: vec!("adjective"),
            adverbs: vec!("adverb"),
            names: vec!("name"),
        };
        assert_eq!(
            petnames.generate(&mut rand::thread_rng(), 3, "-"),
            "adverb-adjective-name");
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
