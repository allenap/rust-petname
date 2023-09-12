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

- Choose from 3 built-in word lists, or provide your own.
- Alliterative names, like _viable-vulture_, _proper-pony_, ...
- Build names with 1-255 components (adjectives, adverbs, nouns).
- Name components can be unseparated, or joined by any character or string.
- Generate 1..n names, or stream names continuously.
- **`no_std` support** (see [later section](#features--no_std-support)).
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
$ petname -h
Generate human readable random names

Usage: petname [OPTIONS]

Options:
  -w, --words <WORDS>             Number of words in name [default: 2]
  -s, --separator <SEP>           Separator between words [default: -]
      --lists <LIST>              Use the built-in word lists with small, medium, or large words [default: small] [possible values: small, medium, large]
  -d, --dir <DIR>                 Use custom word lists by specifying a directory containing `adjectives.txt`, `adverbs.txt`, and `nouns.txt`
      --count <COUNT>             Generate multiple names; or use --stream to generate continuously [default: 1]
      --stream                    Stream names continuously
  -l, --letters <LETTERS>         Maximum number of letters in each word; 0 for unlimited [default: 0]
  -a, --alliterate                Generate names where each word begins with the same letter
  -A, --alliterate-with <LETTER>  Generate names where each word begins with the given letter
  -u, --ubuntu                    Alias; see --alliterate
      --seed <SEED>               Seed the RNG with this value (unsigned 64-bit integer in base-10)
  -h, --help                      Print help (see more with '--help')
  -V, --version                   Print version

Based on Dustin Kirkland's petname project <https://github.com/dustinkirkland/petname>.

$ petname
unified-platypus

$ petname -s _ -w 3
lovely_notable_rooster
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
$ petname --words=3 --stream | grep 'love.*\bsalmon$'
```

## Library

You can use of rust-petname in your own Rust projects with `cargo add petname`.

## Features & `no_std` support

There are a few features that can be selected – or, more correctly,
_deselected_, since all features are enabled by default:

- `default-rng` enables `std` and `std_rng` in [rand][]. A couple of convenience
  functions depend on this for a default RNG.
- `default-words` enables the default word lists. Deselecting this will reduce
  the size of compiled artifacts.
- `clap` enables the [clap][] command-line argument parser, which is needed to
  build the `petname` binary.
  - **NOTE** that `clap` is **not** necessary for the library at all, and you
    can deselect it, but it is presently a default feature since otherwise it's
    inconvenient to build the binary. This will probably change in the future.

All of these are required to build the command-line utility.

The library can be built without any default features, and it will work in a
[`no_std`][no_std] environment, like [Wasm][]. You'll need to figure out a
source of randomness, but [SmallRng::seed_from_u64][smallrng::seed_from_u64] may
be a good starting point.

[rand]: https://crates.io/crates/rand
[clap]: https://crates.io/crates/clap
[no_std]: https://doc.rust-lang.org/reference/crates-and-source-files.html#preludes-and-no_std
[wasm]: https://webassembly.org/
[smallrng::seed_from_u64]: https://docs.rs/rand/latest/rand/trait.SeedableRng.html#method.seed_from_u64

## Upgrading from 1.x

Version 2.0 brought several breaking changes to both the API and the
command-line too. Below are the most important:

### Command-line

- The `--complexity <COMPLEXITY>` option has been replaced by `--lists <LISTS>`.
- When using custom word lists with `--dir <DIR>`, nouns are now found in a file
  named appropriately `DIR/nouns.txt`. Previously this was `names.txt` but this
  was confusing; the term "names" is overloaded enough already.
- The option `--count 0` is no longer a synonym for `--stream`. Use `--stream`
  instead. It's not an error to pass `--count 0`, but it will result in zero
  names being generated.

### Library

- Feature flags have been renamed:
  - `std_rng` is now `default-rng`,
  - `default_dictionary` is now `default-words`.
- The `names` field on the `Petnames` struct has been renamed to `nouns`.
  Previously the complexity was given as a number – 0, 1, or 2 – but now the
  word lists to use are given as a string: small, medium, or large.
- `Petnames::new()` is now `Petnames::default()`.
- `Petnames::new(…)` now accepts word lists as strings.
- `Names` is no longer public. This served as the iterator struct returned by
  `Petnames::iter(…)`, but this now hides the implementation details by
  returning `impl Iterator<Item = String>` instead. This also means that
  `Names::cardinality(&self)` is no longer available; use
  `Petnames::cardinality(&self, words: u8)` instead.

## Developing & Contributing

To hack the source:

- [Install Cargo][install-cargo],
- Clone this repository,
- Build it: `cargo build`.
- Optionally, hide noise when using `git blame`: `git config blame.ignoreRevsFile .git-blame-ignore-revs`.

[install-cargo]: https://crates.io/install

### Running the tests

After installing the source (see above) run tests with: `cargo test`.

### Making a release

1. Bump version in [`Cargo.toml`](Cargo.toml).
2. Paste updated `-h` output into [`README.md`](README.md) (this file; see near
   the top). On macOS the command `cargo run -- -h | pbcopy` is helpful.
   **Note** that `--help` output is not the same as `-h` output: it's more
   verbose and too much for an overview.
3. Build **and** test: `cargo build && cargo test`. The latter on its own does
   do a build, but a test build can hide warnings about dead code, so do both.
4. Commit with message "Bump version to `$VERSION`."
5. Tag with "v`$VERSION`", e.g. `git tag v1.0.10`.
6. Push: `git push --follow-tags`.
7. Publish: `cargo publish`.

## License

This project is licensed under the Apache 2.0 License. See the
[LICENSE](LICENSE) file for details.
