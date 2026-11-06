//! Finishing a reservation: things come back, material is used.
//!
//! Pattern: State (use) — `finish` of a thing adds to `done` and ends when all are back. Material
//! ends at once with `done` = what was used; the stock is derived from that (see `stock`), so
//! the only write is the reservation and a failed `finish` changes nothing.

use fs_store::ObjectId;

use super::handle::ResourceRepo;
use crate::error::{Error, Result};
use crate::model::{Reservation, ReservationState, ResourceKind};
use crate::state::{Step, reservation_next};

impl ResourceRepo {
    pub(super) async fn finish_reservation(
        &self,
        id: ObjectId,
        quantity: u32,
    ) -> Result<Reservation> {
        if quantity == 0 {
            return Err(Error::Invalid("the quantity must not be zero"));
        }
        let mut r = self.require_reservation(id).await?;
        let ended = reservation_next(r.state, Step::Finish)?;
        if quantity > r.quantity.saturating_sub(r.done) {
            return Err(Error::Conflict("more than is out"));
        }
        let resource = self.require_resource(r.resource).await?;
        if resource.kind == ResourceKind::Material {
            if quantity > resource.quantity {
                return Err(Error::Conflict("more than is in stock"));
            }
            r.done = quantity;
            r.state = ended;
        } else {
            r.done += quantity;
            r.state = if r.done == r.quantity {
                ended
            } else {
                ReservationState::Confirmed
            };
        }
        self.save(&r).await?;
        Ok(r)
    }
}
