//! [`ResourceRepo`]: the store-backed implementation of the resources traits.
//!
//! Pattern: Repository + Facade — one small type hides the object store and the envelope rules
//! behind `Items`, `Reservations`, `Capacities`, `Needs` and `ResourceFeed`. Its collaborators are
//! injected as the narrow store traits, so each can be swapped or wrapped (Decorator in tests).

use std::sync::Arc;

use fs_store::traits::{ChangeFeed, ObjectQuery, ObjectRepository, UnloggedObjects};
use tokio::sync::Mutex;

use crate::traits::{Capacities, GroupLookup, Items, Needs, Reservations};

/// Resources over an object store. Cheap to clone; clones share one write lock.
#[derive(Clone)]
pub struct ResourceRepo {
    pub(super) objects: Arc<dyn ObjectRepository>,
    pub(super) reads: Arc<dyn ObjectQuery>,
    pub(super) private: Arc<dyn UnloggedObjects>,
    pub(super) private_reads: Arc<dyn ObjectQuery>,
    pub(super) events: Arc<dyn ChangeFeed>,
    pub(super) groups: Option<Arc<dyn GroupLookup>>,
    pub(super) actor: String,
    pub(super) instance: String,
    pub(super) page_size: usize,
    /// Serialises every write that reads first (confirm, change, set quantity, accepting an offer,
    /// capacity), so two cannot overbook or overlap together.
    pub(super) lock: Arc<Mutex<()>>,
}

impl ResourceRepo {
    /// Resources over a store, with capacity, allocations and shares in a store of their own
    /// (`private`): they never touch `store`, its log or its feed. `actor` is the IRI of the acting
    /// person: the log of public objects names it, and every `Capacities` method acts for it.
    pub fn new<S, P>(store: Arc<S>, private: Arc<P>, actor: impl Into<String>) -> Self
    where
        S: ObjectRepository + ObjectQuery + ChangeFeed + 'static,
        P: UnloggedObjects + ObjectQuery + 'static,
    {
        Self {
            objects: store.clone(),
            reads: store.clone(),
            private: private.clone(),
            private_reads: private,
            events: store,
            groups: None,
            actor: actor.into(),
            instance: "local.invalid".into(),
            page_size: 100,
            lock: Arc::new(Mutex::new(())),
        }
    }

    /// The groups program, for `group_load`. Without it that answers `Unsupported`.
    #[must_use]
    pub fn with_groups(mut self, groups: Arc<dyn GroupLookup>) -> Self {
        self.groups = Some(groups);
        self
    }

    /// Instance domain for external IRIs `https://<instance>/resources/<id>` (ADR 0027).
    #[must_use]
    pub fn with_instance(mut self, instance: impl Into<String>) -> Self {
        self.instance = instance.into();
        self
    }

    /// Items per page (default 100, at least 1).
    #[must_use]
    pub fn with_page_size(mut self, n: usize) -> Self {
        self.page_size = n.max(1);
        self
    }

    /// The `Items` interface of this provider.
    pub fn items(&self) -> &dyn Items {
        self
    }

    /// The `Reservations` interface of this provider.
    pub fn reservations(&self) -> &dyn Reservations {
        self
    }

    /// The `Capacities` interface of this provider; acts for the actor.
    pub fn capacity(&self) -> &dyn Capacities {
        self
    }

    /// The `Needs` interface of this provider.
    pub fn needs(&self) -> &dyn Needs {
        self
    }
}
