#[macro_use]
extern crate clap;

mod lib;
use lib::Petnames;

use std::collections::HashSet;
use std::fmt;
use std::fs;
use std::io::{self, Write};
use std::path;
use std::process;
use std::str::FromStr;

use clap::Arg;
use rand::seq::IteratorRandom;

fn main() {
    let matches = app().get_matches();
    match run(matches) {
        Err(Error::Disconnected) => {
            process::exit(0);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
        Ok(()) => {
            process::exit(0);
        }
    }
}

#[derive(Debug)]
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
            Error::Io(ref e) => write!(f, "{}", e),
            Error::FileIo(ref path, ref e) => write!(f, "{}: {}", e, path.display()),
            Error::Cardinality(ref message) => write!(f, "cardinality is zero: {}", message),
            Error::Alliteration(ref message) => write!(f, "cannot alliterate: {}", message),
            Error::Disconnected => write!(f, "caller disconnected / stopped reading"),
        }
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error::Io(error)
    }
}

fn app<'a, 'b>() -> clap::App<'a, 'b> {
    clap::App::new("rust-petname")
        .version(crate_version!())
        .author(crate_authors!())
        .about("Generate human readable random names.")
        .after_help(concat!(
            "Based on Dustin Kirkland's petname project ",
            "<https://github.com/dustinkirkland/petname>."
        ))
        .arg(
            Arg::with_name("words")
                .short("w")
                .long("words")
                .value_name("WORDS")
                .default_value("2")
                .help("Number of words in name")
                .takes_value(true)
                .validator(can_be_parsed::<u8>),
        )
        .arg(
            Arg::with_name("separator")
                .short("s")
                .long("separator")
                .value_name("SEP")
                .default_value("-")
                .help("Separator between words")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("complexity")
                .short("c")
                .long("complexity")
                .value_name("COM")
                .possible_values(&["0", "1", "2"])
                .hide_possible_values(true)
                .default_value("0")
                .help("Use small words (0), medium words (1), or large words (2)")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("directory")
                .short("d")
                .long("dir")
                .value_name("DIR")
                .help("Directory containing adjectives.txt, adverbs.txt, names.txt")
                .conflicts_with("complexity")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("count")
                .long("count")
                .value_name("COUNT")
                .default_value("1")
                .help("Generate multiple names; pass 0 to produce infinite names!")
                .takes_value(true)
                .validator(can_be_parsed::<usize>),
        )
        .arg(
            Arg::with_name("letters")
                .short("l")
                .long("letters")
                .value_name("LETTERS")
                .default_value("0")
                .help("Maxiumum number of letters in each word; 0 for unlimited")
                .takes_value(true)
                .validator(can_be_parsed::<usize>),
        )
        .arg(
            Arg::with_name("alliterate")
                .short("a")
                .long("alliterate")
                .help("Generate names where each word begins with the same letter")
                .takes_value(false),
        )
        .arg(
            // For compatibility with upstream.
            Arg::with_name("ubuntu")
                .short("u")
                .long("ubuntu")
                .help("Alias; see --alliterate")
                .takes_value(false),
        )
}

fn run(matches: clap::ArgMatches) -> Result<(), Error> {
    // Unwrapping is safe because these options have defaults.
    let opt_separator = matches.value_of("separator").unwrap();
    let opt_words = matches.value_of("words").unwrap();
    let opt_complexity = matches.value_of("complexity").unwrap();
    let opt_count = matches.value_of("count").unwrap();
    let opt_letters = matches.value_of("letters").unwrap();
    let opt_alliterate = matches.is_present("alliterate") || matches.is_present("ubuntu");

    // Optional arguments without defaults.
    let opt_directory = matches.value_of("directory");

    // Parse numbers. Validated so unwrapping is okay.
    let opt_words: u8 = opt_words.parse().unwrap();
    let opt_count: usize = opt_count.parse().unwrap();
    let opt_letters: usize = opt_letters.parse().unwrap();

    // Load custom word lists, if specified.
    let words = match opt_directory {
        Some(dirname) => Words::load(dirname)?,
        None => Words::Builtin,
    };

    // Select the appropriate word list.
    let mut petnames = match words {
        Words::Custom(ref adjectives, ref adverbs, ref names) => {
            Petnames::init(&adjectives, &adverbs, &names)
        }
        Words::Builtin => match opt_complexity {
            "0" => Petnames::small(),
            "1" => Petnames::medium(),
            "2" => Petnames::large(),
            _ => Petnames::small(),
        },
    };

    // If requested, limit the number of letters.
    if opt_letters != 0 {
        petnames.retain(|s| s.len() <= opt_letters);
    }

    // Check cardinality.
    if petnames.cardinality(opt_words) == 0 {
        return Err(Error::Cardinality(
            "no petnames to choose from; try relaxing constraints".to_string(),
        ));
    }

    // We're going to need a source of randomness.
    let mut rng = rand::thread_rng();

    // If requested, choose a random letter then discard all words that do not
    // begin with that letter.
    if opt_alliterate {
        // We choose the first letter from the intersection of the first letters
        // of each word list in `petnames`.
        let firsts =
            common_first_letters(&petnames.adjectives, &[&petnames.adverbs, &petnames.names]);
        // Choose the first letter at random; fails if there are no letters.
        match firsts.iter().choose(&mut rng) {
            Some(c) => petnames.retain(|s| s.starts_with(*c)),
            None => {
                return Err(Error::Alliteration(
                    "word lists have no initial letters in common".to_string(),
                ))
            }
        };
    }

    // Manage stdout.
    let stdout = io::stdout();
    let mut writer = io::BufWriter::new(stdout.lock());

    let (s, r) = crossbeam::channel::bounded(4 * 10);
    let _ = crossbeam::scope(|scope| {
        let mut handles = Vec::new();
        for _ in 0..4 {
            handles.push(scope.spawn(|_| {
                let mut rng = rand::thread_rng();
                // Get an iterator for the names we want to print out.
                let names = petnames.iter(&mut rng, opt_words, opt_separator);
                for name in names {
                    if let Err(_) = s.send(name) {
                        break;
                    }
                }
            }))
        }

        // Stream if count is 0.
        if opt_count == 0 {
            for name in r.iter() {
                writeln!(writer, "{}", name)
                    .map_err(suppress_disconnect)
                    .unwrap();
            }
        } else {
            for name in r.iter().take(opt_count) {
                writeln!(writer, "{}", name).unwrap();
            }
        }

        drop(r);

        for handle in handles.drain(..) {
            handle.join().unwrap();
        }
    })
    .unwrap();

    Ok(())
}

fn can_be_parsed<INTO>(value: String) -> Result<(), String>
where
    INTO: FromStr,
    <INTO as FromStr>::Err: std::fmt::Display,
{
    match value.parse::<INTO>() {
        Err(e) => Err(format!("{}", e)),
        Ok(_) => Ok(()),
    }
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
    // files in that directory: `adjectives.txt`, `adverbs.txt`, and `names.txt`.
    // Each should be valid UTF-8, and contain words separated by whitespace.
    fn load<T: AsRef<path::Path>>(dirname: T) -> Result<Self, Error> {
        let dirname = dirname.as_ref();
        Ok(Self::Custom(
            read_file_to_string(dirname.join("adjectives.txt"))?,
            read_file_to_string(dirname.join("adverbs.txt"))?,
            read_file_to_string(dirname.join("names.txt"))?,
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
