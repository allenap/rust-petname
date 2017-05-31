extern crate rand;

use self::rand::Rng;


pub enum WordKind {Adjective, Adverb, Name}
pub enum ListKind {Large, Medium, Small}

pub use self::WordKind::*;
pub use self::ListKind::*;

pub struct WordList<'a> {
    pub wordkind: WordKind,
    pub listkind: ListKind,
    words: Vec<&'a str>,
}

impl<'a> WordList<'a> {

    pub fn load(wordkind: WordKind, listkind: ListKind) -> WordList<'a> {
        let wordlist = match wordkind {
            Adjective => match listkind {
                Large => include_str!("../words/large/adjectives.txt"),
                Medium => include_str!("../words/medium/adjectives.txt"),
                Small => include_str!("../words/small/adjectives.txt"),
            },
            Adverb => match listkind {
                Large => include_str!("../words/large/adverbs.txt"),
                Medium => include_str!("../words/medium/adverbs.txt"),
                Small => include_str!("../words/small/adverbs.txt"),
            },
            Name => match listkind {
                Large => include_str!("../words/large/names.txt"),
                Medium => include_str!("../words/medium/names.txt"),
                Small => include_str!("../words/small/names.txt"),
            },
        };
        WordList{
            wordkind: wordkind, listkind: listkind,
            words: wordlist.split_whitespace().collect(),
        }
    }

    pub fn random(&self) -> &'a str {
        rand::thread_rng().choose(&self.words).unwrap()
    }

    pub fn iter(&self) -> WordListIter {
        WordListIter{wordlist: self, index: 0}
    }

    pub fn iter_random(&self) -> WordListRandomIter {
        WordListRandomIter{wordlist: self}
    }

    pub fn len(&self) -> usize {
        self.words.len()
    }

    pub fn is_empty(&self) -> bool {
        self.words.is_empty()
    }

}

pub struct WordListIter<'a> {
    wordlist: &'a WordList<'a>,
    index: usize,
}

impl<'a> Iterator for WordListIter<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.wordlist.len() {
            let next = Some(self.wordlist.words[self.index]);
            self.index += 1;
            next
        }
        else {
            None
        }
    }
}

pub struct WordListRandomIter<'a> {
    wordlist: &'a WordList<'a>,
}

impl<'a> Iterator for WordListRandomIter<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.wordlist.random())
    }
}


#[cfg(test)]
mod tests {

    use super::{WordList, WordKind, ListKind};

    #[test]
    fn large_adjectives_is_not_empty() {
        let list = WordList::load(
            WordKind::Adjective, ListKind::Large);
        assert_ne!(list.len(), 0);
        assert!(!list.is_empty());
    }

    #[test]
    fn large_adverbs_is_not_empty() {
        let list = WordList::load(
            WordKind::Adverb, ListKind::Large);
        assert_ne!(list.len(), 0);
        assert!(!list.is_empty());
    }

    #[test]
    fn large_names_is_not_empty() {
        let list = WordList::load(
            WordKind::Name, ListKind::Large);
        assert_ne!(list.len(), 0);
        assert!(!list.is_empty());
    }

    #[test]
    fn medium_adjectives_is_not_empty() {
        let list = WordList::load(
            WordKind::Adjective, ListKind::Medium);
        assert_ne!(list.len(), 0);
        assert!(!list.is_empty());
    }

    #[test]
    fn medium_adverbs_is_not_empty() {
        let list = WordList::load(
            WordKind::Adverb, ListKind::Medium);
        assert_ne!(list.len(), 0);
        assert!(!list.is_empty());
    }

    #[test]
    fn medium_names_is_not_empty() {
        let list = WordList::load(
            WordKind::Name, ListKind::Medium);
        assert_ne!(list.len(), 0);
        assert!(!list.is_empty());
    }

    #[test]
    fn small_adjectives_is_not_empty() {
        let list = WordList::load(
            WordKind::Adjective, ListKind::Small);
        assert_ne!(list.len(), 0);
        assert!(!list.is_empty());
    }

    #[test]
    fn small_adverbs_is_not_empty() {
        let list = WordList::load(
            WordKind::Adverb, ListKind::Small);
        assert_ne!(list.len(), 0);
        assert!(!list.is_empty());
    }

    #[test]
    fn small_names_is_not_empty() {
        let list = WordList::load(
            WordKind::Name, ListKind::Small);
        assert_ne!(list.len(), 0);
        assert!(!list.is_empty());
    }

}
