#![feature(let_chains)]

use std::path::PathBuf;

use anyhow::bail;
use clap::Parser;

use cargo_unify::{expand, extend_path, read_path};

// cargo invokes this subcommand as `cargo-unify unify ...`,
// thus we define this single enum variant.
#[derive(Parser, Debug)]
#[command(about = "A tool to unify crates into one buildable file")]
enum Cli {
    Unify {
        /// If set, a lib crate will be unified.
        #[arg(long, default_value_t = false)]
        lib: bool,

        /// If set, a bin crate will be unified (default).
        #[arg(long, default_value_t = false)]
        bin: bool,

        /// Path to the crate root (i.e., where the `src` is). If not set, will default to current dir.
        #[arg(long, default_value = ".")]
        path: PathBuf,
    },
}

impl Cli {
    pub fn validate(&self) -> anyhow::Result<()> {
        let &Self::Unify { lib, bin, .. } = self;
        if bin && lib {
            bail!("Cannot set both --lib and --bin")
        }
        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    let args = Cli::parse();
    args.validate()?;

    let Cli::Unify { lib, path, .. } = args;

    let file_name = if !lib { "main.rs" } else { "lib.rs" };
    let path = extend_path(&path, &["src", file_name]);

    let expanded = expand(&read_path(&path)?, &path)?;

    println!("{}", expanded);

    Ok(())
}
