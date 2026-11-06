//! Reservations and free periods.
//!
//! Pattern: Value object — plain data; the state rules live in `state`, the arithmetic in `sweep`.

use fs_store::{ObjectId, Timestamp};
use serde::{Deserialize, Serialize};

use super::record::Record;

named_enum! {
    /// Where a reservation is in its life.
    ReservationState {
        /// Holds nothing yet.
        Requested = "requested",
        /// Holds its quantity for its time; material from `from` until it is finished.
        Confirmed = "confirmed",
        /// Ended; for material the stock is now lower by `done`.
        Returned = "returned",
        /// Withdrawn by the one who asked.
        Cancelled = "cancelled",
        /// Refused by the one who decides.
        Declined = "declined",
    }
}

/// A reservation of a quantity of a resource for a time `[from, to)`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Reservation {
    /// Id (UUIDv7).
    pub id: ObjectId,
    /// The resource.
    pub resource: ObjectId,
    /// How many.
    pub quantity: u32,
    /// Start (included).
    pub from: Timestamp,
    /// End (excluded).
    pub to: Timestamp,
    /// Person or group IRI.
    pub holder: String,
    /// Where it stands.
    pub state: ReservationState,
    /// IRI of the task or event it is for; empty = none.
    pub subject: String,
    /// Free note.
    pub note: String,
    /// Thing, room: how many came back so far. Material: how many were used.
    pub done: u32,
}

/// One stretch of time in which the same quantity is free.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FreePeriod {
    /// Start (included).
    pub from: Timestamp,
    /// End (excluded).
    pub to: Timestamp,
    /// How many are free throughout.
    pub free: u32,
}

impl Record for Reservation {
    const TYPE: &'static str = "Reservation";
    fn id(&self) -> ObjectId {
        self.id
    }
    fn name(&self) -> String {
        format!("Reservation of {}", self.holder)
    }
    fn index(&self) -> Vec<(&'static str, String)> {
        vec![
            ("resource", self.resource.to_string()),
            ("holder", self.holder.clone()),
            ("state", self.state.as_str().into()),
            ("subject", self.subject.clone()),
        ]
    }
}
