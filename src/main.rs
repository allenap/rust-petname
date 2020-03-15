#[macro_use]
extern crate clap;

mod lib;
use lib::Petnames;

use std::collections::HashSet;
use std::fs;
use std::io;
use std::path;
use std::process;
use std::str::FromStr;

use clap::{App, Arg};
use rand::seq::IteratorRandom;

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

fn run() -> io::Result<()> {
    let matches = App::new("rust-petname")
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
        .get_matches();

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
            Some(c) => petnames.retain(|s| s.chars().next() == Some(*c)),
            None => panic!("no letters in common"), // TODO: do this without a panic.
        };
    }

    // Stream if count is 0.
    if opt_count == 0 {
        loop {
            let petname = petnames.generate(&mut rng, opt_words, opt_separator);
            println!("{}", petname);
        }
    } else {
        for _ in 1..=opt_count {
            let petname = petnames.generate(&mut rng, opt_words, opt_separator);
            println!("{}", petname);
        }
    }

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
    fn load<T: AsRef<path::Path>>(dirname: T) -> io::Result<Self> {
        let dirname = dirname.as_ref();
        Ok(Self::Custom(
            fs::read_to_string(dirname.join("adjectives.txt"))?,
            fs::read_to_string(dirname.join("adverbs.txt"))?,
            fs::read_to_string(dirname.join("names.txt"))?,
        ))
    }
}
