//! The window of `fs-resources` (ADR 0063): wiring only.
//!
//! Pattern: Facade consumer — parse arguments, open the two stores, build the window, show it. The
//! runtime lives as long as the window: the backend's calls run on it.

use std::sync::Arc;

use fs_components_app::{WindowOptions, launch};
use fs_interfaces::rpc::{default_data_dir, prepare_data_dir};
use fs_objects::TursoStore;
use fs_resources_core::ResourceRepo;

use crate::app_widget;
use crate::args::{HELP, parse};
use crate::backend::Ports;

/// Who acts on these stores. Provisional: there is no sign-in yet (same as the other programs).
const ACTOR: &str = "https://localhost/actors/owner";

/// Open the window. `words` are the words without the program name.
///
/// # Errors
/// The message for the person: bad words, a store that does not open, a window that cannot start.
pub fn run(words: &[String]) -> Result<(), String> {
    let args = parse(words)?;
    if args.help {
        println!("{HELP}");
        return Ok(());
    }
    let runtime = tokio::runtime::Runtime::new().map_err(|e| e.to_string())?;
    let repo = runtime.block_on(open(&args))?;
    let app = app_widget(&Ports::from_repo(&repo), runtime.handle()).map_err(|e| e.to_string())?;
    let (width, height) = args.size;
    if let Some(file) = &args.snapshot {
        app.picture(width, height, file)
            .map_err(|e| e.to_string())?;
        eprintln!("wrote {}", file.display());
        return Ok(());
    }
    let mut options = WindowOptions::default();
    options.width = width;
    options.height = height;
    launch(options, app).map_err(|e| e.to_string())
}

/// The main store and the private one (capacity never reaches the hub), in the data folder or in memory.
async fn open(args: &crate::args::Args) -> Result<ResourceRepo, String> {
    let (main, private) = if args.memory || args.snapshot.is_some() && args.data.is_none() {
        (
            TursoStore::open_in_memory().await,
            TursoStore::open_in_memory().await,
        )
    } else {
        let dir = args
            .data
            .clone()
            .or_else(|| default_data_dir("resources"))
            .ok_or("no data folder: give --data or set $XDG_DATA_HOME or $HOME")?;
        let dir = prepare_data_dir(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        (
            TursoStore::open(dir.join("objects.db")).await,
            TursoStore::open(dir.join("private.db")).await,
        )
    };
    let main = main.map_err(|e| format!("cannot open the store: {e}"))?;
    let private = private.map_err(|e| format!("cannot open the private store: {e}"))?;
    Ok(ResourceRepo::new(Arc::new(main), Arc::new(private), ACTOR))
}
