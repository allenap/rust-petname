# rust-petname

A [petname][petname-intro] library and command-line tool in [Rust][rust-lang].
Dustin Kirkland's [petname][] project is the inspiration for this project. The
word lists and command-line UX here are taken from that project. Check it out!
Dustin also maintains libraries for [Python 2 & 3][petname-py], and
[Golang][petname-go].

[rust-lang]: https://www.rust-lang.org/
[petname-intro]: http://blog.dustinkirkland.com/2015/01/introducing
[petname]: https://github.com/dustinkirkland/petname
[petname-py]: https://pypi.org/project/petname/
[petname-go]: https://github.com/dustinkirkland/golang-petname


## Command-line utility

```
$ petname --help
rust-petname 1.0.0
Gavin Panella <gavinpanella@gmail.com>
Generate human readable random names.

USAGE:
    petname [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -s, --separator <SEP>    Separator between words [default: -]
    -w, --words <WORDS>      Number of words in name [default: 2]

Based on Dustin Kirkland's petname project <https://github.com/dustinkirkland/petname>.

$ petname
untaunting-paxton

$ petname -s _ -w 3
suitably_overdelicate_jamee
```


## Getting Started

To install the command-line tool:

  * [Install cargo](https://crates.io/install),
  * Install this crate: `cargo install petname`.

Alternatively, to hack the source:

  * [Install cargo](https://crates.io/install),
  * Clone this repository,
  * Build it: `cargo build`.


## Running the tests

After installing the source (see above) run tests with: `cargo test`.


## License

This project is licensed under the Apache 2.0 License. See the
[LICENSE](LICENSE) file for details.
