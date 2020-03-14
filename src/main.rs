#[macro_use]
extern crate clap;

mod lib;

use clap::{App, Arg};
use lib::Petnames;
use std::io::Write;
use std::process;

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
                .takes_value(true),
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
        .get_matches();

    // Unwrapping is safe because these options have defaults.
    let opt_separator = matches.value_of("separator").unwrap();
    let opt_words = matches.value_of("words").unwrap();
    let opt_complexity = matches.value_of("complexity").unwrap();

    // Parse the words option into a number.
    let opt_words: u8 = opt_words.parse().unwrap_or_else(|error| {
        writeln!(
            std::io::stderr(),
            "--words={} could not be parsed: {}",
            opt_words,
            error
        )
        .ok();
        process::exit(1);
    });

    // Select the appropriate word list.
    let petnames = match opt_complexity {
        "0" => Petnames::small(),
        "1" => Petnames::medium(),
        "2" => Petnames::large(),
        _ => Petnames::small(),
    };

    println!("{}", petnames.generate_one(opt_words, opt_separator));
}
