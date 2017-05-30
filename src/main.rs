mod lib;
use lib::{adjective,adverb,name};


fn main() {
    println!("Hello, {}-{}-{}!", adverb(), adjective(), name());
}
