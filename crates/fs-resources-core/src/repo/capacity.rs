//! Capacity: set, remove, read mine, share with a group.
//!
//! Pattern: Repository (private) — every record is the caller's own and is written without the
//! log. A record of another person answers `Unknown`, exactly like an id that does not exist.

use fs_store::{ObjectId, Query, Timestamp};

use super::handle::ResourceRepo;
use super::reservation_ops::check_range;
use crate::error::{Error, Result};
use crate::model::{Capacity, GroupShare};
use crate::sweep::overlaps;

impl ResourceRepo {
    /// The caller's own capacity record, or `Unknown`.
    async fn my_capacity(&self, id: ObjectId) -> Result<Capacity> {
        let found = self.load::<Capacity>(id).await?;
        found
            .filter(|c| c.person == self.actor)
            .ok_or(Error::Unknown("capacity"))
    }

    pub(super) async fn set_capacity(&self, wanted: Capacity) -> Result<Capacity> {
        check_range(wanted.from, wanted.to)?;
        if wanted.minutes_per_week == 0 {
            return Err(Error::Invalid("the minutes per week must not be zero"));
        }
        let mine = self.capacities_of(&self.actor).await?;
        // A given id is kept only if the record is mine; in every other case, taken or free, the
        // record gets a fresh id, so `set` says nothing about ids of others.
        let id = if mine.iter().any(|m| m.id == wanted.id) {
            wanted.id
        } else {
            ObjectId::new()
        };
        let clash = mine
            .iter()
            .any(|m| m.id != id && overlaps(m.from, m.to, wanted.from, wanted.to));
        if clash {
            return Err(Error::Conflict("the range overlaps another capacity"));
        }
        let saved = Capacity {
            id,
            person: self.actor.clone(),
            ..wanted
        };
        self.save(&saved).await?;
        Ok(saved)
    }

    pub(super) async fn remove_capacity(&self, id: ObjectId) -> Result<()> {
        self.my_capacity(id).await?;
        self.erase::<Capacity>(id).await
    }

    pub(super) async fn capacity_in(
        &self,
        from: Timestamp,
        to: Timestamp,
    ) -> Result<Vec<Capacity>> {
        check_range(from, to)?;
        let mut found = self.capacities_of(&self.actor).await?;
        found.retain(|c| overlaps(c.from, c.to, from, to));
        found.sort_by_key(|c| (c.from, c.id));
        Ok(found)
    }

    pub(super) async fn share(&self, group: ObjectId, share: bool) -> Result<()> {
        let q = Query::new()
            .with_index("person", self.actor.clone())
            .with_index("group", group.to_string());
        let existing = self.find::<GroupShare>(q).await?;
        // One share per person and group; a duplicate is removed, and stopping removes them all.
        let keep = usize::from(share);
        for s in existing.iter().skip(keep) {
            self.erase::<GroupShare>(s.id).await?;
        }
        if share && existing.is_empty() {
            let s = GroupShare {
                id: ObjectId::new(),
                person: self.actor.clone(),
                group,
            };
            self.save(&s).await?;
        }
        Ok(())
    }

    pub(super) async fn shared_with(&self) -> Result<Vec<ObjectId>> {
        let q = Query::new().with_index("person", self.actor.clone());
        let mut groups: Vec<_> = self
            .find::<GroupShare>(q)
            .await?
            .into_iter()
            .map(|s| s.group)
            .collect();
        groups.sort();
        Ok(groups)
    }
}
