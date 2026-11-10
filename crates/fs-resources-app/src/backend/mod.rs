//! The `Backend` of the admin frontend over the resources core.
//!
//! Pattern: Facade — only declarations and re-exports. `core_backend.rs` has the `Backend` (Repository),
//! one module per data type has its commands (Strategy).

mod allocation;
mod capacity;
mod core_backend;
mod errors;
mod input;
mod need;
mod offer;
mod out;
mod ports;
mod reservation;
mod resource;

pub use core_backend::{CoreBackend, unsupported_refusal};
pub use ports::Ports;
