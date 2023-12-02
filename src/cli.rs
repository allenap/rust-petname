use std::path::PathBuf;

use clap::{builder::PossibleValue, Parser};

/// Generate human readable random names.
#[derive(Parser)]
#[command(
    name = "rust-petname",
    version,
    author,
    after_help = "Based on Dustin Kirkland's petname project <https://github.com/dustinkirkland/petname>."
)]
pub struct Cli {
    /// Number of words in name
    #[arg(short, long, value_name = "WORDS", default_value_t = 2)]
    pub words: u8,

    /// Separator between words
    #[arg(short, long, value_name = "SEP", default_value = "-")]
    pub separator: String,

    /// Use the built-in word lists with small, medium, or large words
    #[arg(long, value_name = "LIST", default_value_t = WordList::Medium)]
    pub lists: WordList,

    // For compatibility with upstream.
    /// Alias for compatibility with upstream; prefer --lists instead
    #[arg(short, long, value_name = "NUM", default_value = None, conflicts_with = "lists", hide_possible_values = true)]
    pub complexity: Option<WordList>,

    /// Use custom word lists by specifying a directory containing
    /// `adjectives.txt`, `adverbs.txt`, and `nouns.txt`
    #[arg(short, long = "dir", value_name = "DIR", conflicts_with = "lists")]
    pub directory: Option<PathBuf>,

    /// Generate multiple names; or use --stream to generate continuously
    #[arg(long, value_name = "COUNT", default_value_t = 1)]
    pub count: usize,

    /// Stream names continuously
    #[arg(long, conflicts_with = "count")]
    pub stream: bool,

    /// Maximum number of letters in each word; 0 for unlimited
    #[arg(short, long, value_name = "LETTERS", default_value_t = 0)]
    pub letters: usize,

    /// Generate names where each word begins with the same letter
    #[arg(short, long)]
    pub alliterate: bool,

    /// Generate names where each word begins with the given letter
    #[arg(short = 'A', long, value_name = "LETTER")]
    pub alliterate_with: Option<char>,

    // For compatibility with upstream.
    /// Alias for compatibility with upstream; prefer --alliterate instead
    #[arg(short, long, conflicts_with = "alliterate", conflicts_with = "alliterate_with")]
    pub ubuntu: bool,

    /// Seed the RNG with this value (unsigned 64-bit integer in base-10)
    ///
    /// This makes the names chosen deterministic and repeatable: with the same
    /// seed, the same names will be emitted. Note that which name or names are
    /// emitted is not guaranteed across versions of rust-petname because the
    /// underlying random number generator in use explicitly does not make that
    /// guarantee.
    #[arg(long, value_name = "SEED")]
    pub seed: Option<u64>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum WordList {
    Small,
    Medium,
    Large,
}

impl std::fmt::Display for WordList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Small => write!(f, "small"),
            Self::Medium => write!(f, "medium"),
            Self::Large => write!(f, "large"),
        }
    }
}

impl clap::ValueEnum for WordList {
    fn value_variants<'a>() -> &'a [Self] {
        &[Self::Small, Self::Medium, Self::Large]
    }

    fn to_possible_value(&self) -> Option<PossibleValue> {
        // Numeric aliases and the `-c|--complexity` alias for `--lists` are
        // for compatibility with https://github.com/dustinkirkland/petname.
        Some(match self {
            Self::Small => PossibleValue::new("small").alias("0"),
            Self::Medium => PossibleValue::new("medium").alias("1"),
            Self::Large => PossibleValue::new("large").alias("2"),
        })
    }
}

/// Ensure command-line argument compatibility with those supported in Dustin
/// Kirkland's [`petname`](https://github.com/dustinkirkland/petname). The
/// documentation strings from the tests below are taken verbatim from that
/// project's README.
#[cfg(test)]
mod compatibility {
    use clap::Parser;

    use super::{Cli, WordList};

    /// -w|--words number of words in the name, default is 2,
    #[test]
    fn compat_words() {
        assert_eq!(Cli::parse_from(["petname"]).words, 2);
        assert_eq!(Cli::parse_from(["petname", "-w", "7"]).words, 7);
        assert_eq!(Cli::parse_from(["petname", "--words", "5"]).words, 5);
        assert_eq!(Cli::parse_from(["petname", "--words=6"]).words, 6);
    }

    /// -l|--letters maximum number of letters in each word, default is
    /// unlimited,
    #[test]
    fn compat_letters() {
        assert_eq!(Cli::parse_from(["petname"]).letters, 0); // means: unlimited.
        assert_eq!(Cli::parse_from(["petname", "-l", "7"]).letters, 7);
        assert_eq!(Cli::parse_from(["petname", "--letters", "5"]).letters, 5);
        assert_eq!(Cli::parse_from(["petname", "--letters=6"]).letters, 6);
    }

    /// -s|--separator string used to separate name words, default is '-',
    #[test]
    fn compat_separator() {
        assert_eq!(Cli::parse_from(["petname"]).separator, "-");
        assert_eq!(Cli::parse_from(["petname", "-s", ":"]).separator, ":");
        assert_eq!(Cli::parse_from(["petname", "--separator", "|"]).separator, "|");
        assert_eq!(Cli::parse_from(["petname", "--separator=."]).separator, ".");
    }

    /// -c|--complexity [0, 1, 2]; 0 = easy words, 1 = standard words, 2 =
    /// complex words, default=1,
    #[test]
    fn compat_complexity() {
        assert_eq!(Cli::parse_from(["petname"]).complexity, None);
        assert_eq!(Cli::parse_from(["petname", "-c", "0"]).complexity, Some(WordList::Small));
        assert_eq!(Cli::parse_from(["petname", "--complexity", "1"]).complexity, Some(WordList::Medium));
        assert_eq!(Cli::parse_from(["petname", "--complexity=2"]).complexity, Some(WordList::Large));
    }

    /// -u|--ubuntu generate ubuntu-style names, alliteration of first character
    /// of each word.
    #[test]
    fn compat_ubuntu() {
        assert!(!Cli::parse_from(["petname"]).ubuntu);
        assert!(Cli::parse_from(["petname", "-u"]).ubuntu);
        assert!(Cli::parse_from(["petname", "--ubuntu"]).ubuntu);
    }
}
