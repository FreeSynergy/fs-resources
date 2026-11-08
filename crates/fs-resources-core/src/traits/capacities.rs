//! The [`Capacities`] contract: a person's own time. Every method acts for the calling person.
//!
//! Pattern: Repository (interface) + Template method + Null Object — as [`Items`](super::Items).
//! Another person's record answers exactly like an unknown id.

use async_trait::async_trait;
use fs_objects::{ObjectId, Timestamp};

use crate::error::Result;
use crate::model::{Allocation, Capacity, GroupLoad, Load};
use crate::unsupported::not_possible;

/// Capacity, allocations and sharing of the calling person.
#[async_trait]
pub trait Capacities: Send + Sync {
    /// `capacity.read`: my capacity that touches `[from, to)`, by start.
    async fn mine(&self, from: Timestamp, to: Timestamp) -> Result<Vec<Capacity>>;
    /// `capacity.read`: allocations asked of me that touch `[from, to)`.
    async fn allocations(&self, from: Timestamp, to: Timestamp) -> Result<Vec<Allocation>>;
    /// `allocation.propose`: allocations I asked for, with the answer.
    async fn proposed(&self, from: Timestamp, to: Timestamp) -> Result<Vec<Allocation>>;
    /// `capacity.read`: my capacity and accepted allocations over `[from, to)`.
    async fn load(&self, from: Timestamp, to: Timestamp) -> Result<Load>;
    /// `capacity.share`: the groups I share with.
    async fn shared_groups(&self) -> Result<Vec<ObjectId>>;

    /// `capacity.set`: create or replace; the person is always me; a range may not overlap another.
    async fn set(&self, _capacity: Capacity) -> Result<Capacity> {
        not_possible("capacity.set")
    }
    /// `capacity.set`: remove one of mine.
    async fn remove(&self, _id: ObjectId) -> Result<()> {
        not_possible("capacity.set")
    }
    /// `allocation.propose`: anyone may ask; the answer is always `proposed` and does not reveal
    /// whether the person exists.
    async fn propose(
        &self,
        _person: &str,
        _subject: &str,
        _from: Timestamp,
        _to: Timestamp,
        _minutes: u32,
    ) -> Result<Allocation> {
        not_possible("allocation.propose")
    }
    /// `allocation.answer`: only the person; can be repeated, so an acceptance can be taken back.
    async fn answer(&self, _id: ObjectId, _accept: bool) -> Result<Allocation> {
        not_possible("allocation.answer")
    }
    /// `allocation.propose`: only who proposed it.
    async fn withdraw(&self, _id: ObjectId) -> Result<Allocation> {
        not_possible("allocation.propose")
    }
    /// `capacity.share`: share my load with one group, or stop. Off until the person says so.
    async fn share_with_group(&self, _group: ObjectId, _share: bool) -> Result<()> {
        not_possible("capacity.share")
    }
    /// `capacity.group`: rounded totals of the people who share with this group; hidden below the
    /// minimum.
    async fn group_load(
        &self,
        _group: ObjectId,
        _from: Timestamp,
        _to: Timestamp,
    ) -> Result<GroupLoad> {
        not_possible("capacity.group")
    }
}
