//! A person's load over a range.
//!
//! Pattern: Specification (use) — reads one person's capacity and accepted allocations and hands
//! them to the pure pro rata functions. Private to this crate: callers outside ask only for
//! themselves (`Capacities::load`) or for a group total in whole weeks.

use fs_objects::{Query, Timestamp};

use super::handle::ResourceRepo;
use crate::error::Result;
use crate::model::{Allocation, Capacity, Load};
use crate::sweep::{allocated_minutes, capacity_minutes};

/// One person's capacity records and accepted allocations.
pub(super) struct Records {
    caps: Vec<Capacity>,
    allocs: Vec<Allocation>,
}

impl Records {
    pub(super) fn load(&self, from: Timestamp, to: Timestamp) -> Load {
        Load {
            capacity_minutes: capacity_minutes(&self.caps, from, to),
            allocated_minutes: allocated_minutes(&self.allocs, from, to),
        }
    }
}

impl ResourceRepo {
    pub(super) async fn capacities_of(&self, person: &str) -> Result<Vec<Capacity>> {
        self.find(Query::new().with_index("person", person)).await
    }

    pub(super) async fn records_of(&self, person: &str) -> Result<Records> {
        let allocs = self
            .find(
                Query::new()
                    .with_index("person", person)
                    .with_index("state", "accepted"),
            )
            .await?;
        Ok(Records {
            caps: self.capacities_of(person).await?,
            allocs,
        })
    }

    pub(super) async fn load_of(
        &self,
        person: &str,
        from: Timestamp,
        to: Timestamp,
    ) -> Result<Load> {
        Ok(self.records_of(person).await?.load(from, to))
    }
}
