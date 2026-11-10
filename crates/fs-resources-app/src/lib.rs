//! `fs-resources-app`: the resources program as a window (ADR 0065, 0066).
//!
//! The window is the shell of `fs-components-app` around the generated Admin frontend of
//! `fs-admin-frontend`; the frontend is made from the type descriptions of the core. This crate holds
//! no render code and no look: a [`CoreBackend`] per data type (records and commands over the core),
//! the words, the arguments and the wiring.
//!
//! Pattern: Facade — this file only declares modules and re-exports the public API.

#![deny(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(
    clippy::must_use_candidate,
    clippy::module_name_repetitions,
    clippy::missing_errors_doc
)]

pub mod args;
mod backend;
pub mod gui;
mod window;
pub mod words;

pub use backend::{CoreBackend, Ports, unsupported_refusal};
pub use window::{ABOUT_ORIGIN, app_widget, views};
