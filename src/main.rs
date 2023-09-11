mod cli;

use cli::Cli;
use petname::Petnames;

use std::collections::HashSet;
use std::fmt;
use std::fs;
use std::io;
use std::path;
use std::process;

use clap::Parser;
use rand::{seq::IteratorRandom, SeedableRng};

fn main() {
    let cli = Cli::parse();
    match run(cli) {
        Ok(()) | Err(Error::Disconnected) => {
            process::exit(0);
        }
        Err(e) => {
            eprintln!("Error: {e}");
            process::exit(1);
        }
    }
}

enum Error {
    Io(io::Error),
    FileIo(path::PathBuf, io::Error),
    Cardinality(String),
    Alliteration(String),
    Disconnected,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Error::Io(ref e) => write!(f, "{e}"),
            Error::FileIo(ref path, ref e) => write!(f, "{e}: {}", path.display()),
            Error::Cardinality(ref message) => write!(f, "cardinality is zero: {message}"),
            Error::Alliteration(ref message) => write!(f, "cannot alliterate: {message}"),
            Error::Disconnected => write!(f, "caller disconnected / stopped reading"),
        }
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error::Io(error)
    }
}

fn run(cli: Cli) -> Result<(), Error> {
    // Load custom word lists, if specified.
    let words = match cli.directory {
        Some(dirname) => Words::load(dirname)?,
        None => Words::Builtin,
    };

    // Select the appropriate word list.
    let mut petnames = match words {
        Words::Custom(ref adjectives, ref adverbs, ref nouns) => Petnames::new(adjectives, adverbs, nouns),
        Words::Builtin => match cli.complexity {
            cli::Complexity::Small => Petnames::small(),
            cli::Complexity::Medium => Petnames::medium(),
            cli::Complexity::Large => Petnames::large(),
        },
    };

    // If requested, limit the number of letters.
    let letters = cli.letters;
    if letters != 0 {
        petnames.retain(|s| s.len() <= letters);
    }

    // Check cardinality.
    if petnames.cardinality(cli.words) == 0 {
        return Err(Error::Cardinality("no petnames to choose from; try relaxing constraints".to_string()));
    }

    // We're going to need a source of randomness.
    let mut rng =
        cli.seed.map(rand::rngs::StdRng::seed_from_u64).unwrap_or_else(rand::rngs::StdRng::from_entropy);

    // Handle alliteration, either by eliminating a specified
    // character, or using a random one.
    let alliterate = cli.alliterate || cli.ubuntu || cli.alliterate_with.is_some();
    if alliterate {
        // We choose the first letter from the intersection of the
        // first letters of each word list in `petnames`.
        let firsts = common_first_letters(&petnames.adjectives, &[&petnames.adverbs, &petnames.nouns]);
        // if a specific character was requested for alliteration,
        // attempt to use it.
        if let Some(c) = cli.alliterate_with {
            if firsts.contains(&c) {
                petnames.retain(|s| s.starts_with(c));
            } else {
                return Err(Error::Alliteration(
                    "no petnames begin with the chosen alliteration character".to_string(),
                ));
            }
        } else {
            // Otherwise choose the first letter at random; fails if
            // there are no letters.
            match firsts.iter().choose(&mut rng) {
                Some(c) => petnames.retain(|s| s.starts_with(*c)),
                None => {
                    return Err(Error::Alliteration(
                        "word lists have no initial letters in common".to_string(),
                    ))
                }
            };
        }
    }

    // Manage stdout.
    let stdout = io::stdout();
    let mut writer = io::BufWriter::new(stdout.lock());

    // Stream, or print a limited number of words?
    let count = if cli.stream { None } else { Some(cli.count) };

    // Get an iterator for the names we want to print out.
    if cli.non_repeating {
        printer(&mut writer, petnames.iter_non_repeating(&mut rng, cli.words, &cli.separator), count)
    } else {
        printer(&mut writer, petnames.iter(&mut rng, cli.words, &cli.separator), count)
    }
}

fn printer<OUT, NAMES>(writer: &mut OUT, names: NAMES, count: Option<usize>) -> Result<(), Error>
where
    OUT: io::Write,
    NAMES: Iterator<Item = String>,
{
    match count {
        None => {
            for name in names {
                writeln!(writer, "{name}").map_err(suppress_disconnect)?;
            }
        }
        Some(n) => {
            for name in names.take(n) {
                writeln!(writer, "{name}")?;
            }
        }
    }

    Ok(())
}

fn common_first_letters(init: &[&str], more: &[&[&str]]) -> HashSet<char> {
    let mut firsts = first_letters(init);
    let firsts_other: Vec<HashSet<char>> = more.iter().map(|list| first_letters(list)).collect();
    firsts.retain(|c| firsts_other.iter().all(|fs| fs.contains(c)));
    firsts
}

fn first_letters(names: &[&str]) -> HashSet<char> {
    names.iter().filter_map(|s| s.chars().next()).collect()
}

enum Words {
    Custom(String, String, String),
    Builtin,
}

impl Words {
    // Load word lists from the given directory. This function expects to find three
    // files in that directory: `adjectives.txt`, `adverbs.txt`, and `nouns.txt`.
    // Each should be valid UTF-8, and contain words separated by whitespace.
    fn load<T: AsRef<path::Path>>(dirname: T) -> Result<Self, Error> {
        let dirname = dirname.as_ref();
        Ok(Self::Custom(
            read_file_to_string(dirname.join("adjectives.txt"))?,
            read_file_to_string(dirname.join("adverbs.txt"))?,
            read_file_to_string(dirname.join("nouns.txt"))?,
        ))
    }
}

fn read_file_to_string<P: AsRef<path::Path>>(path: P) -> Result<String, Error> {
    fs::read_to_string(&path).map_err(|error| Error::FileIo(path.as_ref().to_path_buf(), error))
}

fn suppress_disconnect(err: io::Error) -> Error {
    match err.kind() {
        io::ErrorKind::BrokenPipe => Error::Disconnected,
        _ => err.into(),
    }
}
