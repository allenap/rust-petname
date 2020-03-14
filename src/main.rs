#[macro_use]
extern crate clap;

mod lib;

use clap::{App, Arg};
use lib::Petnames;

fn main() {
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
                .validator(is_u8),
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
                .value_name("COMPLEXITY")
                .possible_values(&["0", "1", "2"])
                .hide_possible_values(true)
                .default_value("0")
                .help("Use small words (0), medium words (1), or large words (2)")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("count")
                .long("count")
                .value_name("COUNT")
                .default_value("1")
                .help("Generate multiple names. Set to 0 to produce infinite names!")
                .takes_value(true)
                .validator(is_u64),
        )
        .get_matches();

    // Unwrapping is safe because these options have defaults.
    let opt_separator = matches.value_of("separator").unwrap();
    let opt_words = matches.value_of("words").unwrap();
    let opt_complexity = matches.value_of("complexity").unwrap();
    let opt_count = matches.value_of("count").unwrap();

    // Parse words and count into numbers. Validated so unwrapping is okay.
    let opt_words: u8 = opt_words.parse().unwrap();
    let opt_count: u64 = opt_count.parse().unwrap();

    // Select the appropriate word list.
    let petnames = match opt_complexity {
        "0" => Petnames::small(),
        "1" => Petnames::medium(),
        "2" => Petnames::large(),
        _ => Petnames::small(),
    };

    // We're going to need a source of randomness.
    let mut rng = rand::thread_rng();

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
}

fn is_u8(value: String) -> Result<(), String> {
    match value.parse::<u8>() {
        Err(e) => Err(format!("{}", e)),
        Ok(_) => Ok(()),
    }
}

fn is_u64(value: String) -> Result<(), String> {
    match value.parse::<u64>() {
        Err(e) => Err(format!("{}", e)),
        Ok(_) => Ok(()),
    }
}
