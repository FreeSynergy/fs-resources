//! From the errors of the core to what the frontend can show.
//!
//! Pattern: Adapter (error mapping). `BackendError` has three shapes. Not found and unknown are
//! `NotFound`; a rule of the core that refused (overbooked, wrong state, overlapping range) is
//! `Rejected` with no finding, because no `RecordProblem` says "the program's rule refused" (limit,
//! DESIGN.md); the reason goes to the log so it is not lost. The store failing is `Unavailable`.

use fs_admin_frontend::BackendError;
use fs_resources_core::Error;

/// Map a core error.
pub fn from_core(e: Error) -> BackendError {
    match e {
        Error::NotFound { .. } | Error::Unknown(_) => BackendError::NotFound,
        Error::Store(_) | Error::Json(_) => BackendError::Unavailable(e.to_string()),
        Error::Invalid(_)
        | Error::Conflict(_)
        | Error::NotFree { .. }
        | Error::InvalidTransition { .. }
        | Error::Unsupported(_) => {
            eprintln!("fs-resources: {e}");
            BackendError::Rejected(Vec::new())
        }
    }
}

/// Shorthand for `map_err(from_core)`.
pub trait CoreResult<T> {
    /// The result with the error mapped.
    fn shown(self) -> Result<T, BackendError>;
}

impl<T> CoreResult<T> for Result<T, Error> {
    fn shown(self) -> Result<T, BackendError> {
        self.map_err(from_core)
    }
}
