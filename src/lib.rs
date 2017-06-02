#![allow(dead_code)]

extern crate rand;


pub fn petname(words: u16, separator: &str) -> String {
    let petnames = Petnames::default();
    let mut rng = rand::thread_rng();
    generate(&petnames, &mut rng, words, separator)
}


pub fn generate<RNG>(
    petnames: &Petnames, rng: &mut RNG,
    words: u16, separator: &str) -> String
    where RNG: rand::Rng
{
    // Adverbs all the way, finishing with adjective then name.
    let mut parts = Vec::with_capacity(words as usize);
    for num in (0..words).rev() {
        parts.push(*match num {
            0 => rng.choose(&petnames.names).unwrap(),
            1 => rng.choose(&petnames.adjectives).unwrap(),
            _ => rng.choose(&petnames.adverbs).unwrap(),
        });
    };
    parts.join(separator)
}


pub struct Petnames<'a> {
    pub adjectives: Vec<&'a str>,
    pub adverbs: Vec<&'a str>,
    pub names: Vec<&'a str>,
}

impl<'a> Petnames<'a> {

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

}


#[cfg(test)]
mod tests {

    use super::{generate, Petnames, rand};

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
            generate(&petnames, &mut rand::thread_rng(), 3, "-"),
            "adverb-adjective-name");
    }

}
