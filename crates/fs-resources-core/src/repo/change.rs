//! Changing a reservation in one step.
//!
//! Pattern: State (use) — a requested reservation just takes the new values; a confirmed one is
//! checked against the other holds with its new values *instead of* its old ones, in a single write,
//! so it keeps what it holds or nothing changes.

use fs_store::{ObjectId, Timestamp};

use super::handle::ResourceRepo;
use super::reservation_ops::check_range;
use crate::error::{Error, Result};
use crate::model::{Reservation, ReservationState};

impl ResourceRepo {
    pub(super) async fn change_reservation(
        &self,
        id: ObjectId,
        quantity: u32,
        from: Timestamp,
        to: Timestamp,
    ) -> Result<Reservation> {
        if quantity == 0 {
            return Err(Error::Invalid("the quantity must not be zero"));
        }
        check_range(from, to)?;
        let old = self.require_reservation(id).await?;
        if self.held_by_offer(id).await? {
            return Err(Error::Conflict(
                "an accepted offer holds it; withdraw the offer",
            ));
        }
        let new = Reservation {
            quantity,
            from,
            to,
            ..old.clone()
        };
        match old.state {
            ReservationState::Requested => {}
            ReservationState::Confirmed => {
                if old.done > 0 {
                    return Err(Error::Conflict("part of it has already come back"));
                }
                let resource = self.require_resource(old.resource).await?;
                self.ensure_fits(&resource, &new).await?;
            }
            other => {
                return Err(Error::InvalidTransition {
                    what: "reservation",
                    from: other.as_str(),
                    step: "changed",
                });
            }
        }
        self.save(&new).await?;
        Ok(new)
    }
}
