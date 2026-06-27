use std::borrow::Cow;
use std::path::{Path, PathBuf};

use syn::LitStr;

use crate::input;

/// Paths resolved from input to the macros in this crate.
///
/// These can be relative paths, because they have not yet been anchored to
/// `CARGO_MANIFEST_DIR` for example, or absolute, like after a call to
/// [`Self::resolve`].
pub struct PetnamesPaths {
    pub adjectives: PathBuf,
    pub adverbs: PathBuf,
    pub nouns: PathBuf,
}

impl From<input::PetnamesInput> for PetnamesPaths {
    fn from(input: input::PetnamesInput) -> Self {
        fn value_or<'a>(value: Option<&'_ LitStr>, default: &'a str) -> Cow<'a, str> {
            value.map(LitStr::value).map(Cow::from).unwrap_or_else(|| default.into())
        }

        let path_adjectives = value_or(input.adjectives.as_ref(), "adjectives.txt");
        let path_adverbs = value_or(input.adverbs.as_ref(), "adverbs.txt");
        let path_nouns = value_or(input.nouns.as_ref(), "nouns.txt");

        match input.dir.as_ref().map(LitStr::value).map(PathBuf::from) {
            Some(base) => PetnamesPaths {
                adjectives: base.join(path_adjectives.as_ref()),
                adverbs: base.join(path_adverbs.as_ref()),
                nouns: base.join(path_nouns.as_ref()),
            },
            None => PetnamesPaths {
                adjectives: path_adjectives.as_ref().into(),
                adverbs: path_adverbs.as_ref().into(),
                nouns: path_nouns.as_ref().into(),
            },
        }
    }
}

impl PetnamesPaths {
    pub fn resolve(mut self, path: &Path) -> Self {
        self.adjectives = path.join(self.adjectives);
        self.adverbs = path.join(self.adverbs);
        self.nouns = path.join(self.nouns);
        self
    }
}
