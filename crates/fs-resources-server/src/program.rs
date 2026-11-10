//! `serve`: the server as a command of the one program (ADR 0063).
//!
//! Pattern: Facade — reads its locations, opens the two stores (the main one and the private one
//! that capacity lives in, which never reaches the hub), serves until a signal. The binary
//! `fs-resources` calls it for `fs-resources serve`.

use std::ffi::OsString;
use std::sync::Arc;

use fs_interfaces::rpc::{ServerOptions, prepare_data_dir, run_until_signal};
use fs_objects::TursoStore;
use fs_resources_core::ResourceRepo;

use crate::ResourcesServer;

/// The help text of `fs-resources serve`.
pub const HELP: &str = "fs-resources serve [--socket PATH] [--data DIR] [--actor IRI]\nDefaults: $XDG_RUNTIME_DIR/freesynergy/resources.sock, $XDG_DATA_HOME/freesynergy/resources, the local owner";

/// Why `serve` ended without serving.
#[derive(Debug, PartialEq, Eq)]
pub enum ServeError {
    /// The words were wrong; the caller should exit with 2.
    Usage(String),
    /// The server could not start or stopped with an error; exit with 1.
    Failed(String),
}

/// Serve `resources.capnp` until a signal arrives. `args` are the words after `serve`.
///
/// # Errors
/// [`ServeError::Usage`] for unknown or incomplete words, [`ServeError::Failed`] for everything else.
pub fn serve(args: impl IntoIterator<Item = OsString>) -> Result<(), ServeError> {
    let Some(options) = ServerOptions::parse("resources", args).map_err(ServeError::Usage)? else {
        println!("{HELP}");
        return Ok(());
    };
    let ServerOptions {
        socket,
        data,
        actor,
        instance,
    } = options;
    run_until_signal(&socket, || async move {
        prepare_data_dir(&data)?;
        let main = TursoStore::open(data.join("objects.db"))
            .await
            .map_err(std::io::Error::other)?;
        // Capacity, allocations and shares: a store of their own, never connected to the hub.
        let private = TursoStore::open(data.join("private.db"))
            .await
            .map_err(std::io::Error::other)?;
        let repo = ResourceRepo::new(Arc::new(main), Arc::new(private), actor);
        let repo = match instance {
            Some(domain) => repo.with_instance(domain),
            None => repo,
        };
        Ok(ResourcesServer::new(Arc::new(repo)).into_client().client)
    })
    .map_err(|e| ServeError::Failed(e.to_string()))
}
