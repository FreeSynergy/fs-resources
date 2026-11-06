//! Error type of the resources core.
//!
//! Pattern: none (plain error enum via `thiserror`) — one `Error` for every public operation. The
//! variants a caller must tell apart are named: a rule refused (`Invalid`, `Conflict`), too little
//! free (`NotFree`), a step the state does not allow (`InvalidTransition`), and `Unknown`, the one
//! answer for a private record that is missing *or* belongs to someone else.

use fs_store::ObjectId;

/// Result of every fallible operation of this crate.
pub type Result<T> = std::result::Result<T, Error>;

/// Everything that can go wrong in the resources core.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The object store failed.
    #[error("object store: {0}")]
    Store(#[from] fs_store::Error),
    /// Stored JSON could not be read or written.
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
    /// No public object of this kind with this id.
    #[error("{what} {id} not found")]
    NotFound {
        /// What was looked for (`resource`, `reservation`, `need`, `offer`).
        what: &'static str,
        /// The id that was not found.
        id: ObjectId,
    },
    /// No private record (capacity, allocation) of the caller with that id. Carries no id and no
    /// reason, so a record of someone else answers exactly like one that does not exist.
    #[error("no such {0}")]
    Unknown(&'static str),
    /// The input breaks a rule of the contract (zero quantity, `to <= from`, empty name, ...).
    #[error("refused: {0}")]
    Invalid(&'static str),
    /// The request conflicts with the data (overlap, stale `expected`, still confirmed, ...).
    #[error("refused: {0}")]
    Conflict(&'static str),
    /// Fewer than wanted is free for the whole time asked.
    #[error("refused: {wanted} wanted but only {free} free")]
    NotFree {
        /// How many were asked for.
        wanted: u32,
        /// The lowest quantity that is free in that time.
        free: u32,
    },
    /// The state does not allow this step.
    #[error("{what} is {from} and cannot be {step}")]
    InvalidTransition {
        /// Kind of object (`reservation`, `need`, `offer`, `allocation`).
        what: &'static str,
        /// State it is in.
        from: &'static str,
        /// The step that was tried.
        step: &'static str,
    },
    /// The explicit "not possible" of a provider (ADR 0045).
    #[error(transparent)]
    Unsupported(#[from] fs_interfaces::Unsupported),
}
