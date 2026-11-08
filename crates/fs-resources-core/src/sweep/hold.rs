//! What one reservation holds.
//!
//! Pattern: Specification — a confirmed reservation holds `quantity - done` for `[from, to)`; the
//! hold of material has no end, because what is used on Monday is gone on Tuesday.

use fs_objects::Timestamp;

use crate::model::{Reservation, ReservationState, ResourceKind};

/// The end of a hold that never ends.
pub const FOREVER: Timestamp = Timestamp::MAX_UTC;

/// A quantity held for `[from, to)`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Hold {
    /// Start (included).
    pub from: Timestamp,
    /// End (excluded); [`FOREVER`] for material.
    pub to: Timestamp,
    /// How many are held.
    pub quantity: u32,
}

impl Hold {
    /// The hold of `reservation` on a resource of `kind`; none unless it is confirmed and still out.
    pub fn of(kind: ResourceKind, reservation: &Reservation) -> Option<Self> {
        let out = reservation.quantity.saturating_sub(reservation.done);
        (reservation.state == ReservationState::Confirmed && out > 0).then_some(Self {
            from: reservation.from,
            to: if kind == ResourceKind::Material {
                FOREVER
            } else {
                reservation.to
            },
            quantity: out,
        })
    }

    pub(super) fn covers(&self, t: Timestamp) -> bool {
        self.from <= t && t < self.to
    }
}
