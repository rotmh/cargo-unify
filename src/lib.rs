#![feature(let_chains)]

#[cfg(test)]
mod tests;

use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, anyhow};

macro_rules! extend_path {
    ($base:expr, $($seg:expr),*) => {};
}

/// Recursively expand module declerations.
pub fn expand(file: &str, path: &Path) -> anyhow::Result<String> {
    let dir = mod_dir(path)?;

    let mod_decls = mod_declarations(file).with_context(|| {
        format!(
            "Failed to parse module in: {} for `mod` declarations",
            path.display()
        )
    })?;

    let mut expanded = String::with_capacity(file.len());
    let mut last_match = 0;

    for (name, pos) in mod_decls {
        let (mod_content, mod_path) = get_mod(name.as_str(), &dir)?;

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

/// Finds a module's directory for the module in the provided path.
///
/// If the path file name is `main.rs`, `mod.rs` or `lib.rs`, it is not
/// considered a module, and the module directory is the parent directory
/// of the path. Otherwise, the module directory is the parent directory
/// of the path with the module name appended.
///
/// Examples:
/// `src/main.rs` -> `src/`
/// `src/foo.rs`  -> `src/foo/`
fn mod_dir(path: &Path) -> anyhow::Result<PathBuf> {
    let dir = path
        .parent()
        .with_context(|| format!("Failed to get parent directory of '{}'", path.display()))?;

    let mod_name = path
        .file_stem()
        .and_then(|name| name.to_str())
        .with_context(|| format!("Failed to convert file name '{}' to string", path.display()))?;

    Ok(if ["main", "lib", "mod"].contains(&mod_name) {
        dir.to_path_buf()
    } else {
        dir.join(mod_name)
    })
}

fn get_mod(name: &str, parent: &Path) -> anyhow::Result<(String, PathBuf)> {
    // <name>.rs
    let file_mod = parent.join(name.to_owned() + ".rs");
    // <name>/mod.rs
    let dir_mod = extend_path(parent, &[name, "mod.rs"]);

    for path in [file_mod, dir_mod] {
        if let Ok(content) = read_path(&path) {
            return Ok((content, path));
        }
    }

    Err(anyhow!(
        "Couldn't find module file for module `{name}` in directory `{}`",
        parent.display()
    ))
}

#[inline]
fn read_path(path: &Path) -> anyhow::Result<String> {
    fs::read_to_string(path).with_context(|| format!("Failed to read path {}", path.display()))
}

/// Join `base` with `nodes`.
fn extend_path(base: &Path, nodes: &[&str]) -> PathBuf {
    let mut path = base.to_path_buf();

    for node in nodes {
        path.push(node);
    }

    path
}
