#[macro_use]
extern crate clap;

mod lib;

use lib::petname;
use clap::{Arg, App};
use std::io::Write;
use std::process;


fn main() {
    let matches = App::new("rust-petname")
        .version(crate_version!())
        .author(crate_authors!())
        .about("Generate human readable random names.")
        .after_help(concat!(
            "Based on Dustin Kirkland's petname libraries ",
            "<http://blog.dustinkirkland.com/2015/01/introducing",
            "-petname-libraries-for.html>."))
        .arg(Arg::with_name("words")
             .short("w")
             .long("words")
             .value_name("WORDS")
             .default_value("2")
             .help("Number of words in name")
             .takes_value(true))
        .arg(Arg::with_name("separator")
             .short("s")
             .long("separator")
             .value_name("SEPARATOR")
             .default_value("-")
             .help("Separator between words")
             .takes_value(true))
        .get_matches();

    let opt_separator = matches.value_of("separator").unwrap();
    let opt_words = matches.value_of("words").unwrap();
    let opt_words: u16 = opt_words.parse().unwrap_or_else(|error| {
        writeln!(
            std::io::stderr(), "--words={} could not be parsed: {}",
            opt_words, error).ok();
        process::exit(1);
    });

    println!("{}", petname(opt_words, opt_separator));
}
