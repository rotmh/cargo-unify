mod a;

use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, bail};
use clap::Parser;
use once_cell::sync::Lazy;
use regex::Regex;

#[derive(Parser, Debug)]
struct Cli {
    /// If set, a lib crate will be unified.
    #[arg(long, default_value_t = false)]
    lib: bool,

    /// If set, a bin crate will be unified.
    #[arg(long, default_value_t = true)]
    bin: bool,

    /// Path to the crate root (i.e., where the `src` is). If not set, will default to current dir.
    #[arg(long, default_value = ".")]
    path: PathBuf,
}

impl Cli {
    pub fn validate(&self) -> anyhow::Result<()> {
        if self.bin == self.lib {
            bail!("Cannot set --lib and --bin to the same value");
        }
        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    let args = Cli::parse();

    args.validate()?;

    let mut base = args.path.clone();
    base.push("src");
    base.push(if args.bin { "main.rs" } else { "lib.rs" });

    let content =
        fs::read_to_string(&base).with_context(|| format!("Failed to read {}", &base.display()))?;

    let expanded = expand(&content, &base)?;

    println!("{}", expanded);

    Ok(())
}

/// Recursively expand module declerations.
///
/// # Examples
///
/// ```
/// ```
fn expand(file: &str, path: &PathBuf) -> anyhow::Result<String> {
    // Finds module declerations without definitions. There are two capture
    // groups: 0) the module name, for searching it in the file system.
    // 1) the semicolon, in order to replace it with the module definition.
    static RE: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"(?:pub)?(?:^|\s)[\s]*mod[\s]+([\w]+)[\s]*(;)").unwrap());

    let mut content = file.to_owned();

    for decl in RE.captures_iter(file) {
        let mod_name = decl.get(1).unwrap();

        let Some(parent) = path.parent() else {
            bail!("Failed to get parent dir of {}", path.display());
        };

        let (mod_content, mod_path) = get_mod(mod_name.as_str(), parent)?;

        let mut expanded_mod = expand(&mod_content, &mod_path)?;
        expanded_mod.insert_str(0, " {\n");
        expanded_mod.push_str("\n}");

        let semicolon = decl.get(2).unwrap();
        content.replace_range(semicolon.range(), &expanded_mod);
    }

    Ok(content)
}

fn get_mod(name: &str, parent: &Path) -> anyhow::Result<(String, PathBuf)> {
    // FIXME: search also `<name>/mod.rs` module path.

    let mut mod_path = parent.to_path_buf();

    mod_path.push(name.to_owned() + ".rs");

    let Ok(content) = fs::read_to_string(&mod_path) else {
        bail!("Failed to read path {}", mod_path.display());
    };

    Ok((content, mod_path))
}
