use std::borrow::Cow;

#[cfg(all(feature = "default-rng", feature = "default-words"))]
use petname::petname;
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
        adjectives: Cow::Owned(vec!["adjective"]),
        adverbs: Cow::Owned(vec!["adverb"]),
        nouns: Cow::Owned(vec!["noun"]),
    };
    assert_eq!(petnames.generate(&mut StepRng::new(0, 1), 3, "-"), "adverb-adjective-noun");
}

#[test]
fn petnames_iter_yields_names() {
    let mut rng = StepRng::new(0, 1);
    let petnames = Petnames::new("foo", "bar", "baz");
    let names = petnames.iter(&mut rng, 3, ".");
    // Definitely an Iterator...
    let mut iter: Box<dyn Iterator<Item = _>> = Box::new(names);
    assert_eq!(Some("bar.foo.baz".to_string()), iter.next());
}

#[test]
#[cfg(all(feature = "default-rng", feature = "default-words"))]
fn petname_renders_desired_number_of_words() {
    assert_eq!(petname(7, "-").split('-').count(), 7);
}

#[test]
#[cfg(all(feature = "default-rng", feature = "default-words"))]
fn petname_renders_with_desired_separator() {
    assert_eq!(petname(7, "@").split('@').count(), 7);
}
