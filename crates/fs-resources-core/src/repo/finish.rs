//! Finishing a reservation: things come back, material is used.
//!
//! Pattern: State (use) — `finish` of a thing adds to `done` and ends when all are back. Material
//! lowers the stock by what was used and ends at once. Stock and reservation are two objects and
//! cannot be written in one step; the order is [`FINISH_MATERIAL_ORDER`](super::FINISH_MATERIAL_ORDER):
//! stock first, reservation last. A crash between them leaves a lowered stock with the reservation
//! still `confirmed`, and a second `finish` would lower it again. That limit is stated, not hidden.

use fs_store::ObjectId;

use super::handle::ResourceRepo;
use crate::error::{Error, Result};
use crate::model::{Reservation, ReservationState, Resource, ResourceKind};
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
            let stock = resource
                .quantity
                .checked_sub(quantity)
                .ok_or(Error::Conflict("more than is in stock"))?;
            self.save(&Resource {
                quantity: stock,
                ..resource
            })
            .await?;
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
