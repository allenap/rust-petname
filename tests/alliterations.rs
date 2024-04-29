use std::collections::HashSet;

use rand::rngs::mock::StepRng;

use petname::{Alliterations, Generator, Petnames};

#[test]
fn alliterations_from_petnames() {
    let petnames = Petnames::new("able bold", "burly curly", "ant bee cow");
    let alliterations: Alliterations = petnames.into();
    let alliterations_expected: Alliterations = [
        ('a', Petnames::new("able", "", "ant")),
        ('b', Petnames::new("bold", "burly", "bee")),
        ('c', Petnames::new("", "curly", "cow")),
    ]
    .into();
    assert_eq!(alliterations_expected, alliterations);
}

#[test]
fn alliterations_retain_applies_given_predicate() {
    let petnames = Petnames::new("able bold", "burly curly", "ant bee cow");
    let mut alliterations: Alliterations = petnames.into();
    alliterations.retain(|first_letter, _petnames| *first_letter != 'b');
    let alliterations_expected: Alliterations =
        [('a', Petnames::new("able", "", "ant")), ('c', Petnames::new("", "curly", "cow"))].into();
    assert_eq!(alliterations_expected, alliterations);
}

#[test]
#[cfg(feature = "default-words")]
fn alliterations_default_has_non_zero_cardinality() {
    let alliterations = Alliterations::default();
    // This test will need to be adjusted when word lists change.
    assert_eq!(0, alliterations.cardinality(0));
    assert_eq!(1056, alliterations.cardinality(1));
    assert_eq!(69734, alliterations.cardinality(2));
    assert_eq!(8145549, alliterations.cardinality(3));
    assert_eq!(1137581773, alliterations.cardinality(4));
}

#[test]
fn alliterations_generate_uses_adverb_adjective_name() {
    let petnames = Petnames::new("able bold", "burly curly", "ant bee cow");
    let alliterations: Alliterations = petnames.into();
    assert_eq!(
        alliterations.generate(&mut StepRng::new(1234567890, 1), 3, "-"),
        Some("burly-bold-bee".into())
    );
}

#[test]
fn alliterations_iter_yields_names() {
    let mut rng = StepRng::new(1234567890, 1234567890);
    let petnames = Petnames::new("able bold", "burly curly", "ant bee cow");
    let alliterations: Alliterations = petnames.into();
    let names = alliterations.iter(&mut rng, 3, " ");
    let expected: HashSet<String> = ["able ant", "burly bold bee", "curly cow"].map(String::from).into();
    let observed: HashSet<String> = names.take(10).collect::<HashSet<String>>();
    assert_eq!(expected, observed);
}

#[test]
fn alliterations_iter_yields_nothing_when_empty() {
    let mut rng = StepRng::new(0, 1);
    let alliteration: Alliterations = [].into();
    assert_eq!(0, alliteration.cardinality(3));
    let mut names: Box<dyn Iterator<Item = _>> = alliteration.iter(&mut rng, 3, ".");
    assert_eq!(None, names.next());
}
