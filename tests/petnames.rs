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
    assert_eq!(456, petnames.cardinality(1));
    assert_eq!(204744, petnames.cardinality(2));
    assert_eq!(53438184, petnames.cardinality(3));
    assert_eq!(13947366024, petnames.cardinality(4));
}

#[test]
fn petnames_generate_uses_adverb_adjective_name() {
    let petnames = Petnames {
        adjectives: vec!["adjective"].into(),
        adverbs: vec!["adverb"].into(),
        nouns: vec!["noun"].into(),
    };
    assert_eq!(petnames.generate(&mut StepRng::new(0, 1), 3, "-"), "adverb-adjective-noun");
}

#[test]
fn petnames_iter_yields_names() {
    let mut rng = StepRng::new(0, 1);
    let petnames = Petnames::new("foo", "bar", "baz");
    let mut names: Box<dyn Iterator<Item = _>> = petnames.iter(&mut rng, 3, ".");
    assert_eq!(Some("bar.foo.baz".to_string()), names.next());
}
