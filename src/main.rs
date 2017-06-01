#[macro_use]
extern crate clap;

mod lib;

use lib as petname;
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

    let adjectives = petname::WordList::load(
        petname::Adjective, petname::Large);
    let adverbs = petname::WordList::load(
        petname::Adverb, petname::Large);
    let names = petname::WordList::load(
        petname::Name, petname::Large);

    let opt_words = matches.value_of("words").unwrap();
    let opt_separator = matches.value_of("separator").unwrap();

    let count: u16 = opt_words.parse().unwrap_or_else(|error| {
        writeln!(
            std::io::stderr(), "--words={} could not be parsed: {}",
            opt_words, error).ok();
        process::exit(1);
    });

    let mut words = Vec::with_capacity(count as usize);
    for num in (0..count).rev() {
        words.push(match num {
            0 => names.random(),
            1 => adjectives.random(),
            _ => adverbs.random(),
        });
    }

    let petname = words.join(opt_separator);
    println!("{}", petname);
}
