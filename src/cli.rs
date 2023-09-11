use std::path::PathBuf;

use clap::{Parser, ValueEnum};

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
    #[arg(short, long, value_name = "SIZE", default_value_t = Complexity::Small)]
    pub complexity: Complexity,

    /// Directory containing adjectives.txt, adverbs.txt, nouns.txt
    #[arg(short, long = "dir", value_name = "DIR", conflicts_with = "complexity")]
    pub directory: Option<PathBuf>,

    /// Generate multiple names; or use --stream to generate continuously
    #[arg(long, value_name = "COUNT", default_value_t = 1)]
    pub count: usize,

    /// Stream names continuously
    #[arg(long, conflicts_with = "count")]
    pub stream: bool,

    /// Do not generate the same name more than once
    #[arg(long)]
    pub non_repeating: bool,

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
    /// Alias; see --alliterate
    #[arg(short, long)]
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

#[derive(Clone, ValueEnum)]
pub enum Complexity {
    Small,
    Medium,
    Large,
}

impl std::fmt::Display for Complexity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Small => write!(f, "small"),
            Self::Medium => write!(f, "medium"),
            Self::Large => write!(f, "large"),
        }
    }
}
