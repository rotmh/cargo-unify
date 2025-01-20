#![feature(let_chains)]

use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, anyhow, bail};
use clap::Parser;

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

/// Recursively expand module declerations.
fn expand(file: &str, path: &Path) -> anyhow::Result<String> {
    let dir = path
        .parent()
        .with_context(|| format!("Failed to get parent dir of {}", path.display()))?;

    let mod_decls = mod_declarations(file).with_context(|| {
        format!(
            "Failed to parse module in: {} for `mod` declarations",
            path.display()
        )
    })?;

    let mut expanded = String::with_capacity(file.len());
    let mut last_match = 0;

    for (name, pos) in mod_decls {
        let (mod_content, mod_path) = get_mod(name.as_str(), dir)?;

        let mut expanded_mod = expand(&mod_content, &mod_path)?;
        expanded_mod.insert_str(0, " {\n");
        expanded_mod.push_str("\n} ");

        expanded.push_str(&file[last_match..pos]);
        expanded.push_str(&expanded_mod);

        last_match = pos + 1;
    }

    expanded.push_str(&file[last_match..]);

    Ok(expanded)
}

fn mod_declarations(file: &str) -> anyhow::Result<impl Iterator<Item = (String, usize)>> {
    let ast: syn::File = syn::parse_file(file)?;

    let decls = ast.items.into_iter().filter_map(|item| {
        if let syn::Item::Mod(syn::ItemMod { semi, ident, .. }) = item
            && let Some(syn::token::Semi { spans: [semi] }) = semi
        {
            // Position of the semicolon at the end of the module declaration
            let pos = semi.start();
            let idx = file
                .lines()
                .take(pos.line) // The line is 1-indexed
                .enumerate()
                .fold(
                    pos.line - 1, // to include the line separators
                    |acc, (i, l)| {
                        if i == pos.line - 1 {
                            acc + pos.column
                        } else {
                            acc + l.len()
                        }
                    },
                );

            Some((ident.to_string(), idx))
        } else {
            None
        }
    });

    Ok(decls)
}

fn get_mod(name: &str, parent: &Path) -> anyhow::Result<(String, PathBuf)> {
    // <name>.rs
    let file_mod = extend_path(parent, &[&(name.to_owned() + ".rs")]);
    // <name>/mod.rs
    let dir_mod = extend_path(parent, &[name, "mod.rs"]);

    for path in [file_mod, dir_mod] {
        if let Ok(content) = read_path(&path) {
            return Ok((content, path));
        }
    }

    Err(anyhow!("Couldn't find module file for module `{name}`"))
}

#[inline]
fn read_path(path: &Path) -> anyhow::Result<String> {
    fs::read_to_string(path).with_context(|| format!("Failed to read path {}", path.display()))
}

fn extend_path(base: &Path, nodes: &[&str]) -> PathBuf {
    let mut path = base.to_path_buf();

    for node in nodes {
        path.push(node);
    }

    path
}
