mod cli;

use cli::Cli;
use petname::Alliterations;
use petname::{Generator, Petnames};

use std::fmt;
use std::fs;
use std::io;
use std::path;
use std::process;

use clap::Parser;
use rand::SeedableRng;

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
        Words::Builtin => match cli.lists {
            cli::WordList::Small => Petnames::small(),
            cli::WordList::Medium => Petnames::medium(),
            cli::WordList::Large => Petnames::large(),
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

    // Manage stdout.
    let stdout = io::stdout();
    let mut writer = io::BufWriter::new(stdout.lock());

    // Stream, or print a limited number of words?
    let count = if cli.stream { None } else { Some(cli.count) };

    // Get an iterator for the names we want to print out, handling alliteration.
    if cli.alliterate || cli.ubuntu {
        let mut alliterations: Alliterations = petnames.into();
        alliterations.retain(|_, group| group.cardinality(cli.words) > 0);
        if alliterations.cardinality(cli.words) == 0 {
            return Err(Error::Alliteration("word lists have no initial letters in common".to_string()));
        }
        printer(&mut writer, alliterations.iter(&mut rng, cli.words, &cli.separator), count)
    } else if let Some(alliterate_with) = cli.alliterate_with {
        let mut alliterations: Alliterations = petnames.into();
        alliterations.retain(|first_letter, group| {
            *first_letter == alliterate_with && group.cardinality(cli.words) > 0
        });
        if alliterations.cardinality(cli.words) == 0 {
            return Err(Error::Alliteration(
                "no petnames begin with the chosen alliteration character".to_string(),
            ));
        }
        printer(&mut writer, alliterations.iter(&mut rng, cli.words, &cli.separator), count)
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
            // Load `nouns.txt`, but fall back to trying `names.txt` for
            // compatibility with Dustin Kirkland's _petname_.
            match read_file_to_string(dirname.join("nouns.txt")) {
                Ok(nouns) => nouns,
                Err(err) => match read_file_to_string(dirname.join("names.txt")) {
                    Ok(nouns) => nouns,
                    Err(_) => Err(err)?, // Error from `nouns.txt`.
                },
            },
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
