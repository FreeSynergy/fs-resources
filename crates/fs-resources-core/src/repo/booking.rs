//! What is held on a resource, and whether one more hold fits.
//!
//! Pattern: Specification (use) — loads the confirmed reservations of a resource and hands them to
//! the pure sweep. Callers hold the write lock across this read and their write.

use fs_store::{ObjectId, Query};

use super::handle::ResourceRepo;
use crate::error::{Error, Result};
use crate::model::{Reservation, Resource};
use crate::sweep::{Hold, lowest_free};

impl ResourceRepo {
    pub(super) async fn require_resource(&self, id: ObjectId) -> Result<Resource> {
        let base = self.base_resource(id).await?;
        self.shown(base).await
    }

    pub(super) async fn require_reservation(&self, id: ObjectId) -> Result<Reservation> {
        self.require("reservation", id).await
    }

    /// The holds of all confirmed reservations of `resource`, except the one in `except`.
    pub(super) async fn holds(
        &self,
        resource: &Resource,
        except: Option<ObjectId>,
    ) -> Result<Vec<Hold>> {
        let q = Query::new()
            .with_index("resource", resource.id.to_string())
            .with_index("state", "confirmed");
        Ok(self
            .find::<Reservation>(q)
            .await?
            .iter()
            .filter(|r| Some(r.id) != except)
            .filter_map(|r| Hold::of(resource.kind, r))
            .collect())
    }

    /// `Ok` if `reservation`, taken as confirmed, fits beside the other holds of its resource.
    pub(super) async fn ensure_fits(
        &self,
        resource: &Resource,
        reservation: &Reservation,
    ) -> Result<()> {
        let Some(hold) = Hold::of(resource.kind, reservation) else {
            return Ok(());
        };
        let others = self.holds(resource, Some(reservation.id)).await?;
        let free = lowest_free(resource.quantity, &others, hold.from, hold.to);
        if free < hold.quantity {
            return Err(Error::NotFree {
                wanted: hold.quantity,
                free,
            });
        }
        Ok(())
    }
}
