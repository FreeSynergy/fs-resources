//! The stock of material is derived.
//!
//! Pattern: Specification (derived value) — the stored `quantity` of a resource is its base. What
//! is shown everywhere is the base minus the `done` of the ended reservations of material, so
//! finishing writes the reservation alone and a crash cannot leave stock and reservation apart.
//! Ended reservations of material therefore must not vanish unnoticed: they are only ever deleted
//! together with their resource.

use fs_store::{ObjectId, Query};

use super::handle::ResourceRepo;
use crate::error::Result;
use crate::model::{Reservation, Resource, ResourceKind};

impl ResourceRepo {
    /// Material used up so far: the `done` of its returned reservations.
    pub(super) async fn used(&self, resource: &Resource) -> Result<u32> {
        if resource.kind != ResourceKind::Material {
            return Ok(0);
        }
        let q = Query::new()
            .with_index("resource", resource.id.to_string())
            .with_index("state", "returned");
        let done: u64 = self
            .find::<Reservation>(q)
            .await?
            .iter()
            .map(|r| u64::from(r.done))
            .sum();
        Ok(u32::try_from(done).unwrap_or(u32::MAX))
    }

    /// The resource as shown: base quantity less what was used.
    pub(super) async fn shown(&self, mut resource: Resource) -> Result<Resource> {
        resource.quantity = resource
            .quantity
            .saturating_sub(self.used(&resource).await?);
        Ok(resource)
    }

    /// The stored resource, base quantity as written.
    pub(super) async fn base_resource(&self, id: ObjectId) -> Result<Resource> {
        self.require("resource", id).await
    }
}
