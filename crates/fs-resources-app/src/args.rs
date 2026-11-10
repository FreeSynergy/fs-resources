//! The command line of the window.
//!
//! Pattern: Builder-like parser — plain `std::env::args`, three options.

use std::path::PathBuf;

use fs_components_app::parse_size;

/// What was asked for.
#[derive(Debug, PartialEq, Eq)]
pub struct Args {
    /// Folder for the two stores; `None` = the data folder of the user.
    pub data: Option<PathBuf>,
    /// Keep everything in memory (nothing is written).
    pub memory: bool,
    /// Size of the window or picture.
    pub size: (u32, u32),
    /// Draw a picture to this file instead of opening a window.
    pub snapshot: Option<PathBuf>,
    /// `--help` was given.
    pub help: bool,
}

/// The help text.
pub const HELP: &str = "fs-resources [--data DIR] [--memory] [--size WxH] [--snapshot FILE.png]\nfs-resources serve [--socket PATH] [--data DIR] [--actor IRI]\nDefault data folder: $XDG_DATA_HOME/freesynergy/resources";

/// Parse the words (without the program name).
///
/// # Errors
/// A message for the person.
pub fn parse(words: &[String]) -> Result<Args, String> {
    let mut out = Args {
        data: None,
        memory: false,
        size: (1200, 800),
        snapshot: None,
        help: false,
    };
    let mut it = words.iter();
    while let Some(w) = it.next() {
        let mut value = |name: &str| {
            it.next()
                .cloned()
                .ok_or_else(|| format!("{name} needs a value"))
        };
        match w.as_str() {
            "--data" => out.data = Some(PathBuf::from(value("--data")?)),
            "--memory" => out.memory = true,
            "--size" => out.size = parse_size(&value("--size")?)?,
            "--snapshot" => out.snapshot = Some(PathBuf::from(value("--snapshot")?)),
            "--help" | "-h" => out.help = true,
            other => return Err(format!("unknown word: {other} (try --help)")),
        }
    }
    Ok(out)
}
