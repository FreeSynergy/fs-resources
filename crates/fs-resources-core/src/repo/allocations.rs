//! Allocations: propose, answer, withdraw, read.
//!
//! Pattern: State (use) — every change goes through `allocation_next`. The person answers, the
//! proposer withdraws; anyone else is told `Unknown`, as for an id that does not exist.

use fs_store::{ObjectId, Query, Timestamp};

use super::handle::ResourceRepo;
use super::reservation_ops::check_range;
use crate::error::{Error, Result};
use crate::model::{Allocation, AllocationState};
use crate::state::{AllocationStep, allocation_next};
use crate::sweep::overlaps;

impl ResourceRepo {
    pub(super) async fn new_allocation(
        &self,
        person: &str,
        subject: &str,
        from: Timestamp,
        to: Timestamp,
        minutes: u32,
    ) -> Result<Allocation> {
        if minutes == 0 {
            return Err(Error::Invalid("the minutes must not be zero"));
        }
        if person.is_empty() {
            return Err(Error::Invalid("an allocation needs a person"));
        }
        check_range(from, to)?;
        // Nothing here asks whether the person exists: the answer is the same either way.
        let a = Allocation {
            id: ObjectId::new(),
            person: person.into(),
            subject: subject.into(),
            from,
            to,
            minutes,
            state: AllocationState::Proposed,
            proposer: self.actor.clone(),
        };
        self.save(&a).await?;
        Ok(a)
    }

    pub(super) async fn answer_allocation(&self, id: ObjectId, accept: bool) -> Result<Allocation> {
        let mut a = self
            .load::<Allocation>(id)
            .await?
            .filter(|a| a.person == self.actor)
            .ok_or(Error::Unknown("allocation"))?;
        a.state = allocation_next(a.state, AllocationStep::Answer(accept))?;
        self.save(&a).await?;
        Ok(a)
    }

    pub(super) async fn withdraw_allocation(&self, id: ObjectId) -> Result<Allocation> {
        let mut a = self
            .load::<Allocation>(id)
            .await?
            .filter(|a| a.proposer == self.actor)
            .ok_or(Error::Unknown("allocation"))?;
        a.state = allocation_next(a.state, AllocationStep::Withdraw)?;
        self.save(&a).await?;
        Ok(a)
    }

    /// Allocations where `key` (`person` or `proposer`) is the caller, touching `[from, to)`.
    pub(super) async fn allocations_in(
        &self,
        key: &str,
        from: Timestamp,
        to: Timestamp,
    ) -> Result<Vec<Allocation>> {
        check_range(from, to)?;
        let q = Query::new().with_index(key, self.actor.clone());
        let mut found = self.find::<Allocation>(q).await?;
        found.retain(|a| overlaps(a.from, a.to, from, to));
        found.sort_by_key(|a| (a.from, a.id));
        Ok(found)
    }
}
