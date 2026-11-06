//! Need and offer transitions.
//!
//! Pattern: State — a need is `open` or `covered` depending on its accepted offers, and `closed`
//! for good; an offer is decided once, but an accepted one can still be declined or withdrawn.

use crate::error::{Error, Result};
use crate::model::NeedState::{self, Closed, Covered, Open};
use crate::model::OfferState::{self, Accepted, Declined, Offered, Withdrawn};

/// A step of an offer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OfferStep {
    /// The need's side takes it.
    Accept,
    /// The need's side refuses it, or takes back an acceptance.
    Decline,
    /// The one who offered takes it back.
    Withdraw,
}

/// The state of an offer after `step`, or the refusal.
pub fn offer_next(from: OfferState, step: OfferStep) -> Result<OfferState> {
    match (from, step) {
        (Offered, OfferStep::Accept) => Ok(Accepted),
        (Offered | Accepted, OfferStep::Decline) => Ok(Declined),
        (Offered | Accepted, OfferStep::Withdraw) => Ok(Withdrawn),
        _ => Err(Error::InvalidTransition {
            what: "offer",
            from: from.as_str(),
            step: match step {
                OfferStep::Accept => "accepted",
                OfferStep::Decline => "declined",
                OfferStep::Withdraw => "withdrawn",
            },
        }),
    }
}

/// The state of a need after `close`, or the refusal.
pub fn close_need(from: NeedState) -> Result<NeedState> {
    match from {
        Open | Covered => Ok(Closed),
        Closed => Err(Error::InvalidTransition {
            what: "need",
            from: from.as_str(),
            step: "closed",
        }),
    }
}

/// `covered` once the accepted offers reach `quantity`, `open` again when they fall below it; a
/// closed need stays closed.
pub fn settle(state: NeedState, covered: u32, quantity: u32) -> NeedState {
    match state {
        Closed => Closed,
        _ if covered >= quantity => Covered,
        _ => Open,
    }
}
