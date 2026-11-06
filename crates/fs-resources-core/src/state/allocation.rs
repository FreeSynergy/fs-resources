//! Allocation transitions.
//!
//! Pattern: State — the person can answer again and again (an acceptance can be taken back); the
//! proposer can withdraw until it is declined; `withdrawn` ends it.

use crate::error::{Error, Result};
use crate::model::AllocationState::{self, Accepted, Declined, Proposed, Withdrawn};

/// A step of an allocation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AllocationStep {
    /// The person answers.
    Answer(bool),
    /// The proposer takes it back.
    Withdraw,
}

/// The state after `step`, or the refusal.
pub fn allocation_next(from: AllocationState, step: AllocationStep) -> Result<AllocationState> {
    match (from, step) {
        (Proposed | Accepted | Declined, AllocationStep::Answer(true)) => Ok(Accepted),
        (Proposed | Accepted | Declined, AllocationStep::Answer(false)) => Ok(Declined),
        (Proposed | Accepted, AllocationStep::Withdraw) => Ok(Withdrawn),
        _ => Err(Error::InvalidTransition {
            what: "allocation",
            from: from.as_str(),
            step: match step {
                AllocationStep::Answer(_) => "answered",
                AllocationStep::Withdraw => "withdrawn",
            },
        }),
    }
}
