# rust-petname

Generate human readable random names.

[Petnames][petname-intro] are useful when you need to name a large number of
resources – like servers, services, perhaps bicycles for hire – and you want
those names to be easy to recall and communicate unambiguously. For example,
over a telephone compare saying "please restart remarkably-striking-cricket"
with "please restart s01O97i4": the former is easier to say and less likely to
be misunderstood. Avoiding sequential names adds confidence too: petnames have a
greater lexical distance between them, so errors in transcription can be more
readily detected.

This crate is both a command-line tool and a [Rust][rust-lang] library. Dustin
Kirkland's [petname][] project is the inspiration for this project. The word
lists and the basic command-line UX here are taken from there. Check it out!
Dustin maintains packages for [Python][petname-py], and [Golang][petname-go]
too.

Notable features:

- Choose from 3 built-in word lists, or provide your own;
- Alliterative names, like _viable-vulture_, _proper-pony_, ...;
- Build names with 1-255 components (adjectives, adverbs, nouns);
- Components can be unseparated or joined by and character or string;
- Generate 1..n names, or stream names continuously.
- **`no_std` support** (see [later section](#features--no_std-support));
- Compile without built-in dictionaries to reduce library/binary size.

[rust-lang]: https://www.rust-lang.org/
[petname-intro]: https://blog.dustinkirkland.com/2015/01/introducing-petname-libraries-for.html
[petname]: https://github.com/dustinkirkland/petname
[petname-py]: https://pypi.org/project/petname/
[petname-go]: https://github.com/dustinkirkland/golang-petname

## Command-line utility

If you have [installed Cargo][install-cargo], you can install rust-petname with
`cargo install petname`. This puts a `petname` binary in `~/.cargo/bin`, which
the Cargo installation process will probably have added to your `PATH`.

The `petname` binary from rust-petname is drop-in compatible with the original
`petname`. It's more strict when validating arguments, but for most uses it
should behave the same.

```shellsession
$ petname --help
rust-petname 1.1.1
Gavin Panella <gavinpanella@gmail.com>
Generate human readable random names.

USAGE:
    petname [FLAGS] [OPTIONS]

FLAGS:
    -a, --alliterate    Generate names where each word begins with the same letter
    -h, --help          Prints help information
    -u, --ubuntu        Alias; see --alliterate
    -V, --version       Prints version information

OPTIONS:
    -A, --alliterate-with <LETTER>    Generate names where each word begins with the given letter
    -c, --complexity <COM>            Use small words (0), medium words (1), or large words (2) [default: 0]
        --count <COUNT>               Generate multiple names; pass 0 to produce infinite names! [default: 1]
    -d, --dir <DIR>                   Directory containing adjectives.txt, adverbs.txt, names.txt
    -l, --letters <LETTERS>           Maxiumum number of letters in each word; 0 for unlimited [default: 0]
    -s, --separator <SEP>             Separator between words [default: -]
    -w, --words <WORDS>               Number of words in name [default: 2]

Based on Dustin Kirkland's petname project <https://github.com/dustinkirkland/petname>.

$ petname
untaunting-paxton

$ petname -s _ -w 3
suitably_overdelicate_jamee
```

### Performance

This implementation is considerably faster than the upstream `petname`:

```shellsession
$ time /usr/bin/petname
fit-lark

real    0m0.038s
user    0m0.032s
sys     0m0.008s

$ time target/release/petname
cool-guinea

real    0m0.002s
user    0m0.002s
sys     0m0.000s
```

These timings are irrelevant if you only need to name a single thing, but if you
need to generate 100s or 1000s of names then rust-petname is handy:

```shellsession
$ time { for i in $(seq 1000); do /usr/bin/petname; done; } > /dev/null

real    0m32.058s
user    0m29.360s
sys     0m5.163s

$ time { for i in $(seq 1000); do target/release/petname; done; } > /dev/null

real    0m2.199s
user    0m1.333s
sys     0m0.987s
```

To be fair, `/usr/bin/petname` is a shell script. The Go command-line version
(available from the golang-petname package on Ubuntu) is comparable to the Rust
version for speed, but has very limited options compared to its shell-script
ancestor and to rust-petname.

Lastly, rust-petname has a `--count` option that speeds up generation of names
considerably:

```shellsession
$ time target/release/petname --count=10000000 > /dev/null

real    0m1.327s
user    0m1.322s
sys     0m0.004s
```

That's ~240,000 (two hundred and forty thousand) times faster, for about 7.5
million petnames a second on this hardware. This is useful if you want to apply
an external filter to the names being generated:

```shellsession
$ petname --words=3 --count=0 | grep 'love.*\bsalmon$'
```

## Features & `no_std` support

There are a few features that can be selected – or, more correctly,
_deselected_, since all features are enabled by default:

- `std_rng` enables `std` and `std_rng` in [rand][].
- `default_dictionary` enables the default word lists.
- `clap` enables the [clap][] command-line argument parser.

All of these are required to build the command-line utility.

However, the library can be built without any default features, and it will work
in a [`no_std`][no_std] environment, like [Wasm][]. You'll need to figure out a
source of randomness, but [SmallRng::seed_from_u64][smallrng::seed_from_u64] may
be a good starting point.

[rand]: https://crates.io/crates/rand
[clap]: https://crates.io/crates/clap
[no_std]: https://doc.rust-lang.org/reference/crates-and-source-files.html#preludes-and-no_std
[wasm]: https://webassembly.org/
[smallrng::seed_from_u64]: https://docs.rs/rand/latest/rand/trait.SeedableRng.html#method.seed_from_u64

## Getting Started

To install the command-line tool:

- [Install Cargo][install-cargo],
- Install this crate: `cargo install petname`.

Alternatively, to hack the source:

- [Install Cargo][install-cargo],
- Clone this repository,
- Build it: `cargo build`.

[install-cargo]: https://crates.io/install

## Running the tests

After installing the source (see above) run tests with: `cargo test`.

## Making a release

1. Bump version in [`Cargo.toml`](Cargo.toml).
2. Paste updated `--help` output into [`README.md`](README.md) (this file; see
   near the top). On macOS the command `cargo run -- --help | pbcopy` is
   helpful.
3. Build **and** test: `cargo build && cargo test`. The latter on its own does
   do a build, but a test build can hide warnings about dead code, so do both.
4. Commit with message "Bump version to `$VERSION`."
5. Tag with "v`$VERSION`", e.g. `git tag v1.0.10`.
6. Push: `git push --tags`.
7. Publish: `cargo publish`.

## License

This project is licensed under the Apache 2.0 License. See the
[LICENSE](LICENSE) file for details.
