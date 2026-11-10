//! `fs-resources-server`: the root of `resources.capnp` over a Unix socket, in front of the
//! resources core. It only translates: no resources logic lives here.
//!
//! Interface status: draft (ADR 0045). What is served: `capabilities`, `supportReport`, `types`.
//! What is not: `items`, `reservations`, `capacity`, `needs`, `subscribe` answer Cap'n Proto's
//! "unimplemented"; the capacity part must stay refused until the transport knows the caller
//! (contract header). See `DESIGN.md`, "Unsupported".
//!
//! Pattern: Facade — this file only declares modules and re-exports.

#![deny(missing_docs)]
#![warn(clippy::pedantic)]
// The generated `Server` traits are async for every method, also those that never wait.
#![allow(clippy::missing_errors_doc, clippy::unused_async_trait_impl)]

mod program;
mod root;

pub use program::{HELP, ServeError, serve};
pub use root::{CONTRACT_VERSION, ResourcesServer};
