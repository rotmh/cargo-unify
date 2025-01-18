# `cargo-unify`

A tool to unify crates into one buildable file.

## Usage

### Installation

Clone this repository:

```bash
git clone git@github.com:rotemhoresh/cargo-unify.git
```

Build the project:

```bash
cd cargo-unify
cargo build --release
```

Make sure to move the binary (in `target/release/cargo-unify` to somewhere in your `$PATH`).

### Options

```
      --lib          If set, a lib crate will be unified
      --bin          If set, a bin crate will be unified (default)
      --path <PATH>  Path to the crate root (i.e., where the `src` is). If not set, will default to current dir [default: .]
  -h, --help         Print help
```

### Examples

Example to unify a lib crate, format it using `rustfmt` and write it to a file:

```bash
cargo unify --lib | rustfmt > bundle.rs
```

## Status

I haven't got the time yet for proper testing (only some basic ones on a Linux machine).

### Contributions

PRs, issues, ideas and suggestions and all appreciated and very welcome :)

### License

This project is licenced under [MIT](https://choosealicense.com/licenses/mit/).
