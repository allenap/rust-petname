use petname::{Generator, Petnames};
use rand::rngs::mock::StepRng;

#[test]
#[cfg(feature = "default-words")]
fn petnames_default_has_adjectives() {
    let petnames = Petnames::default();
    assert_ne!(petnames.adjectives.len(), 0);
}

#[test]
#[cfg(feature = "default-words")]
fn petnames_default_has_adverbs() {
    let petnames = Petnames::default();
    assert_ne!(petnames.adverbs.len(), 0);
}

#[test]
#[cfg(feature = "default-words")]
fn petnames_default_has_names() {
    let petnames = Petnames::default();
    assert_ne!(petnames.nouns.len(), 0);
}

#[test]
fn petnames_retain_applies_given_predicate() {
    let petnames_expected = Petnames::new("bob", "bob", "bob jane");
    let mut petnames = Petnames::new("alice bob carol", "alice bob", "bob carol jane");
    petnames.retain(|word| word.len() < 5);
    assert_eq!(petnames_expected, petnames);
}

#[test]
#[cfg(feature = "default-words")]
fn petnames_default_has_non_zero_cardinality() {
    let petnames = Petnames::default();
    // This test will need to be adjusted when word lists change.
    assert_eq!(0, petnames.cardinality(0));
    assert_eq!(1056, petnames.cardinality(1));
    assert_eq!(1265088, petnames.cardinality(2));
    assert_eq!(2062093440, petnames.cardinality(3));
    assert_eq!(3361212307200, petnames.cardinality(4));
}

#[test]
fn petnames_generate_uses_adverb_adjective_name() {
    let petnames = Petnames {
        adjectives: vec!["adjective"].into(),
        adverbs: vec!["adverb"].into(),
        nouns: vec!["noun"].into(),
    };
    assert_eq!(petnames.generate(&mut StepRng::new(0, 1), 3, "-"), Some("adverb-adjective-noun".into()));
}

#[test]
fn petnames_iter_yields_names() {
    let mut rng = StepRng::new(0, 1);
    let petnames = Petnames::new("foo", "bar", "baz");
    let mut names: Box<dyn Iterator<Item = _>> = petnames.iter(&mut rng, 3, ".");
    assert_eq!(Some("bar.foo.baz".to_string()), names.next());
}

#[test]
fn petnames_iter_yields_nothing_when_empty() {
    let mut rng = StepRng::new(0, 1);
    let petnames = Petnames::new("", "", "");
    assert_eq!(0, petnames.cardinality(3));
    let mut names: Box<dyn Iterator<Item = _>> = petnames.iter(&mut rng, 3, ".");
    assert_eq!(None, names.next());
}

#[test]
fn petnames_raw_works() {
    let mut rng = StepRng::new(0, 1);
    let words = [":?-_", "_?:-", "-:_?"];
    let petnames = Petnames::new(words[0], words[1], words[2]);
    let result = petnames.generate_raw(&mut rng, 3).unwrap();
    assert_eq!(3, result.len());
    assert_eq!(vec![words[1], words[0], words[2]], result);
}
