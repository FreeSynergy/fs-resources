//! `ResourceRepo` as a provider of [`Capacities`]. Every method acts for the actor.
//!
//! Pattern: Adapter — writes that read first take the write lock, then call the operation of the
//! same concern.

use async_trait::async_trait;
use fs_objects::{ObjectId, Timestamp};

use super::handle::ResourceRepo;
use super::reservation_ops::check_range;
use crate::error::Result;
use crate::model::{Allocation, Capacity, GroupLoad, Load};
use crate::traits::Capacities;

#[async_trait]
impl Capacities for ResourceRepo {
    async fn mine(&self, from: Timestamp, to: Timestamp) -> Result<Vec<Capacity>> {
        self.capacity_in(from, to).await
    }

    async fn allocations(&self, from: Timestamp, to: Timestamp) -> Result<Vec<Allocation>> {
        self.allocations_in("person", from, to).await
    }

    async fn proposed(&self, from: Timestamp, to: Timestamp) -> Result<Vec<Allocation>> {
        self.allocations_in("proposer", from, to).await
    }

    async fn load(&self, from: Timestamp, to: Timestamp) -> Result<Load> {
        check_range(from, to)?;
        self.load_of(&self.actor, from, to).await
    }

    async fn shared_groups(&self) -> Result<Vec<ObjectId>> {
        self.shared_with().await
    }

    async fn set(&self, capacity: Capacity) -> Result<Capacity> {
        let _write = self.lock.lock().await;
        self.set_capacity(capacity).await
    }

    async fn remove(&self, id: ObjectId) -> Result<()> {
        let _write = self.lock.lock().await;
        self.remove_capacity(id).await
    }

    async fn propose(
        &self,
        person: &str,
        subject: &str,
        from: Timestamp,
        to: Timestamp,
        minutes: u32,
    ) -> Result<Allocation> {
        self.new_allocation(person, subject, from, to, minutes)
            .await
    }

    async fn answer(&self, id: ObjectId, accept: bool) -> Result<Allocation> {
        let _write = self.lock.lock().await;
        self.answer_allocation(id, accept).await
    }

    async fn withdraw(&self, id: ObjectId) -> Result<Allocation> {
        let _write = self.lock.lock().await;
        self.withdraw_allocation(id).await
    }

    async fn share_with_group(&self, group: ObjectId, share: bool) -> Result<()> {
        let _write = self.lock.lock().await;
        self.share(group, share).await
    }

    async fn group_load(
        &self,
        group: ObjectId,
        from: Timestamp,
        to: Timestamp,
    ) -> Result<GroupLoad> {
        self.group_total(group, from, to).await
    }
}
