extern crate rand;


const LARGE_ADJECTIVES: &'static str =
    include_str!("../words/large/adjectives.txt");

const LARGE_ADVERBS: &'static str =
    include_str!("../words/large/adverbs.txt");

const LARGE_NAMES: &'static str =
    include_str!("../words/large/names.txt");


pub fn adjective() -> &'static str {
    random_word(LARGE_ADJECTIVES)
}


pub fn adverb() -> &'static str {
    random_word(LARGE_ADVERBS)
}


pub fn name() -> &'static str {
    random_word(LARGE_NAMES)
}


fn random_word(words: &str) -> &str {
    let words: Vec<&str> = words.split_whitespace().collect();
    words[rand::random::<usize>() % words.len()]
}



#[cfg(test)]
mod tests {

    #[test]
    fn large_adjectives_is_not_empty() {
        assert_ne!(super::LARGE_ADJECTIVES.len(), 0);
    }

    #[test]
    fn large_adverbs_is_not_empty() {
        assert_ne!(super::LARGE_ADVERBS.len(), 0);
    }

    #[test]
    fn large_names_is_not_empty() {
        assert_ne!(super::LARGE_NAMES.len(), 0);
    }

}
