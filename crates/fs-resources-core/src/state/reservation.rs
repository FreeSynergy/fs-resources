//! Reservation transitions.
//!
//! Pattern: State — `requested` goes to `confirmed`, `declined` or `cancelled`; `confirmed` goes to
//! `returned` or `cancelled`; the other three end it.

use crate::error::{Error, Result};
use crate::model::ReservationState::{self, Cancelled, Confirmed, Declined, Requested, Returned};

/// A step of a reservation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    /// Hold the quantity.
    Confirm,
    /// Refuse the request.
    Decline,
    /// Withdraw.
    Cancel,
    /// Everything has come back, or the material is used.
    Finish,
}

impl Step {
    fn past(self) -> &'static str {
        match self {
            Step::Confirm => "confirmed",
            Step::Decline => "declined",
            Step::Cancel => "cancelled",
            Step::Finish => "finished",
        }
    }
}

/// The state after `step`, or the refusal.
pub fn reservation_next(from: ReservationState, step: Step) -> Result<ReservationState> {
    match (from, step) {
        (Requested, Step::Confirm) => Ok(Confirmed),
        (Requested, Step::Decline) => Ok(Declined),
        (Requested | Confirmed, Step::Cancel) => Ok(Cancelled),
        (Confirmed, Step::Finish) => Ok(Returned),
        _ => Err(Error::InvalidTransition {
            what: "reservation",
            from: from.as_str(),
            step: step.past(),
        }),
    }
}
