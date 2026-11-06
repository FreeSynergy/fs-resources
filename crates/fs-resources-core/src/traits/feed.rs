//! The feed contract: live changes of resources, reservations, needs and offers.
//!
//! Pattern: Observer — a [`ResourceListener`] subscribes and is told about every change; dropping
//! the [`Subscription`] ends it (as in `common.capnp`). Capacity, allocations and shares are never
//! reported: they are written without the log, so the store's change feed does not carry them.

use std::sync::Arc;

use fs_store::ObjectId;
use tokio::task::JoinHandle;

/// Which kind of object changed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EntityKind {
    /// A resource.
    Resource,
    /// A reservation.
    Reservation,
    /// A need.
    Need,
    /// An offer.
    Offer,
}

/// The object a change is about.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ResourceEntity {
    /// What kind of object.
    pub kind: EntityKind,
    /// Its id.
    pub id: ObjectId,
}

/// What happened to an object.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResourceChange {
    /// Created.
    Created(ResourceEntity),
    /// Changed.
    Updated(ResourceEntity),
    /// Deleted.
    Deleted(ResourceEntity),
}

/// Receives changes. Called on a task of the runtime: keep it short, hand work off.
pub trait ResourceListener: Send + Sync {
    /// Something changed.
    fn changed(&self, change: &ResourceChange);
}

/// Live updates (`subscribe` of the contract).
pub trait ResourceFeed: Send + Sync {
    /// Tell `listener` about every change until the returned [`Subscription`] is dropped. Must be
    /// called inside a tokio runtime.
    fn subscribe(&self, listener: Arc<dyn ResourceListener>) -> Subscription;
}

/// Keeps a subscription alive; dropping it unsubscribes.
#[must_use = "dropping the subscription ends it"]
pub struct Subscription(JoinHandle<()>);

impl Subscription {
    /// A subscription that runs as `task` and ends it when dropped.
    pub fn new(task: JoinHandle<()>) -> Self {
        Self(task)
    }
}

impl Drop for Subscription {
    fn drop(&mut self) {
        self.0.abort();
    }
}
