#![feature(let_chains)]

use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::Context;
use clap::Parser;

use cargo_unify::expand;

enum CrateType {
    Lib,
    Bin,
}

impl CrateType {
    pub fn base_file(&self, parent: &Path) -> PathBuf {
        let file_name = match self {
            Self::Lib => "lib.rs",
            Self::Bin => "main.rs",
        };

        let mut path = parent.to_path_buf();
        path.push("src");
        path.push(file_name);

        path
    }
}

// cargo invokes this subcommand as `cargo-unify unify ...`,
// thus we define this single enum variant.
#[derive(Parser, Debug)]
#[command(about = "A tool to unify crates into one buildable file")]
enum Cli {
    Unify {
        /// If set, a lib crate will be unified.
        #[arg(long, group = "crate_type")]
        lib: bool,

        /// If set, a bin crate will be unified (default).
        #[arg(long, group = "crate_type")]
        bin: bool,

        /// Path to the crate root (i.e., where the `src` is). If not set, will default to current dir.
        #[arg(long, default_value = ".")]
        path: PathBuf,
    },
}

fn main() -> anyhow::Result<()> {
    let args = Cli::parse();
    let Cli::Unify { lib, path, .. } = args;

    let crate_type = if lib { CrateType::Lib } else { CrateType::Bin };

    let path = crate_type.base_file(&path);
    let content = fs::read_to_string(&path).with_context(|| {
        format!(
            "Failed to read file `{}`.
Maybe you meant to choose another crate type? (`--lib` or `--bin`)
Note: `--bin` is the default.",
            &path.display()
        )
    })?;

    let expanded = expand(&content, &path)?;

    println!("{}", expanded);

    Ok(())
}
