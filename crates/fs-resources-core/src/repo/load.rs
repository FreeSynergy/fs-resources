//! A person's load over a range.
//!
//! Pattern: Specification (use) — reads one person's capacity and accepted allocations and hands
//! them to the pure pro rata functions. Private to this crate: callers outside ask only for
//! themselves (`Capacities::load`) or for a rounded group total.

use fs_store::{Query, Timestamp};

use super::handle::ResourceRepo;
use crate::error::Result;
use crate::model::{Allocation, Capacity, Load};
use crate::sweep::{allocated_minutes, capacity_minutes};

impl ResourceRepo {
    pub(super) async fn capacities_of(&self, person: &str) -> Result<Vec<Capacity>> {
        self.find(Query::new().with_index("person", person)).await
    }

    pub(super) async fn load_of(
        &self,
        person: &str,
        from: Timestamp,
        to: Timestamp,
    ) -> Result<Load> {
        let caps = self.capacities_of(person).await?;
        let allocs: Vec<Allocation> = self
            .find(
                Query::new()
                    .with_index("person", person)
                    .with_index("state", "accepted"),
            )
            .await?;
        Ok(Load {
            capacity_minutes: capacity_minutes(&caps, from, to),
            allocated_minutes: allocated_minutes(&allocs, from, to),
        })
    }
}
