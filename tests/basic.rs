#[cfg(all(feature = "std_rng", feature = "default_dictionary"))]
use petname::petname;
use petname::Petnames;
use rand::rngs::mock::StepRng;

#[test]
#[cfg(feature = "default_dictionary")]
fn default_petnames_has_adjectives() {
    let petnames = Petnames::default();
    assert_ne!(petnames.adjectives.len(), 0);
}

#[test]
#[cfg(feature = "default_dictionary")]
fn default_petnames_has_adverbs() {
    let petnames = Petnames::default();
    assert_ne!(petnames.adverbs.len(), 0);
}

#[test]
#[cfg(feature = "default_dictionary")]
fn default_petnames_has_names() {
    let petnames = Petnames::default();
    assert_ne!(petnames.names.len(), 0);
}

#[test]
fn retain_applies_given_predicate() {
    let petnames_expected = Petnames::init("bob", "bob", "bob jane");
    let mut petnames = Petnames::init("alice bob carol", "alice bob", "bob carol jane");
    petnames.retain(|word| word.len() < 5);
    assert_eq!(petnames_expected, petnames);
}

#[test]
#[cfg(feature = "default_dictionary")]
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
        petnames.generate(&mut StepRng::new(0, 1), 3, "-"),
        "adverb-adjective-name"
    );
}

#[test]
#[cfg(all(feature = "std_rng", feature = "default_dictionary"))]
fn petname_renders_desired_number_of_words() {
    assert_eq!(petname(7, "-").split('-').count(), 7);
}

#[test]
#[cfg(all(feature = "std_rng", feature = "default_dictionary"))]
fn petname_renders_with_desired_separator() {
    assert_eq!(petname(7, "@").split('@').count(), 7);
}

#[test]
fn petnames_iter_has_cardinality() {
    let mut rng = StepRng::new(0, 1);
    let petnames = Petnames::init("a b", "c d e", "f g h i");
    let names = petnames.iter(&mut rng, 3, ".");
    assert_eq!(24u128, names.cardinality());
}

#[test]
fn petnames_iter_yields_names() {
    let mut rng = StepRng::new(0, 1);
    let petnames = Petnames::init("foo", "bar", "baz");
    let names = petnames.iter(&mut rng, 3, ".");
    // Definintely an Iterator...
    let mut iter: Box<dyn Iterator<Item = _>> = Box::new(names);
    assert_eq!(Some("bar.foo.baz".to_string()), iter.next());
}

#[test]
fn petnames_iter_non_repeating_yields_unique_names() {
    let mut rng = StepRng::new(0, 1);
    let petnames = Petnames::init("a1 a2", "b1 b2 b3", "c1 c2");
    let names: Vec<String> = petnames.iter_non_repeating(&mut rng, 3, ".").collect();
    assert_eq!(
        vec![
            "b2.a2.c2", "b3.a2.c2", "b1.a2.c2", "b2.a1.c2", "b3.a1.c2", "b1.a1.c2", "b2.a2.c1",
            "b3.a2.c1", "b1.a2.c1", "b2.a1.c1", "b3.a1.c1", "b1.a1.c1"
        ],
        names
    )
}

#[test]
fn petnames_iter_non_repeating_yields_nothing_when_any_word_list_is_empty() {
    let mut rng = StepRng::new(0, 1);
    let petnames = Petnames::init("a1 a2", "", "c1 c2");
    let names: Vec<String> = petnames.iter_non_repeating(&mut rng, 3, ".").collect();
    assert_eq!(Vec::<String>::new(), names);
}

#[test]
fn petnames_iter_non_repeating_yields_nothing_when_no_word_lists_are_given() {
    let mut rng = StepRng::new(0, 1);
    let petnames = Petnames::init("a1 a2", "b1 b2", "c1 c2");
    let names: Vec<String> = petnames.iter_non_repeating(&mut rng, 0, ".").collect();
    assert_eq!(Vec::<String>::new(), names);
}
