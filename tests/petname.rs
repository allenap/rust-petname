#[cfg(all(feature = "default-rng", feature = "default-words"))]
use petname::petname;

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
