#![allow(dead_code)]

extern crate rand;

use self::rand::Rng;


pub struct Petnamer<'a> {
    pub adjectives: Vec<&'a str>,
    pub adverbs: Vec<&'a str>,
    pub names: Vec<&'a str>,
    pub rng: Box<Rng>,
}

impl<'a> Petnamer<'a> {

    pub fn new() -> Petnamer<'a> {
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
        Petnamer{
            adjectives: adjectives.split_whitespace().collect(),
            adverbs: adverbs.split_whitespace().collect(),
            names: names.split_whitespace().collect(),
            rng: Box::new(rand::thread_rng()),
        }
    }

    pub fn generate(&mut self, words: u16, separator: &str) -> String {
        // Adverbs all the way, finishing with adjective then name.
        let mut parts = Vec::with_capacity(words as usize);
        let ref mut rng = self.rng;
        for num in (0..words).rev() {
            parts.push(*match num {
                0 => rng.choose(&self.names).unwrap(),
                1 => rng.choose(&self.adjectives).unwrap(),
                _ => rng.choose(&self.adverbs).unwrap(),
            });
        };
        // Join the string parts.
        parts.join(separator)
    }

}


pub fn petname(words: u16, separator: &str) -> String {
    Petnamer::new().generate(words, separator)
}



#[cfg(test)]
mod tests {

    #[test]
    fn test() {
    }

}
