//! Shared helpers of the integration tests.

#![allow(dead_code)] // each test file uses a different subset

pub mod failing;

use std::collections::BTreeMap;
use std::sync::Arc;

use async_trait::async_trait;
use fs_resources_core::{
    GroupLookup, Reservation, ReservationState, Resource, ResourceKind, ResourceRepo, Result,
};
use fs_store::{ObjectId, Timestamp, TursoStore};

pub const ME: &str = "https://example.org/people/me";

/// A repository over a fresh in-memory store, and the store itself.
pub async fn setup() -> (ResourceRepo, Arc<TursoStore>) {
    let store = Arc::new(TursoStore::open_in_memory().await.unwrap());
    (repo_for(&store, ME), store)
}

/// Another view of the same store, acting for `actor`.
pub fn repo_for(store: &Arc<TursoStore>, actor: &str) -> ResourceRepo {
    ResourceRepo::new(store.clone(), actor).with_instance("example.org")
}

/// The IRI of person `n`.
pub fn person(n: &str) -> String {
    format!("https://example.org/people/{n}")
}

/// A point in time `h` hours after Monday 2026-03-02 00:00 UTC.
pub fn t(h: i64) -> Timestamp {
    Timestamp::from_timestamp(1_772_409_600 + h * 3600, 0).unwrap()
}

/// A point in time `m` minutes after the same Monday.
pub fn min(m: i64) -> Timestamp {
    Timestamp::from_timestamp(1_772_409_600 + m * 60, 0).unwrap()
}

/// A new resource of `kind` with `quantity`.
pub async fn make(repo: &ResourceRepo, kind: ResourceKind, quantity: u32) -> Resource {
    repo.items()
        .create(kind, "Thing", quantity, "piece", "", "")
        .await
        .unwrap()
}

/// A requested reservation of `quantity` for `[from, to)` hours.
pub async fn ask(
    repo: &ResourceRepo,
    res: &Resource,
    quantity: u32,
    from: i64,
    to: i64,
) -> Reservation {
    repo.reservations()
        .request(res.id, quantity, t(from), t(to), ME, "", "")
        .await
        .unwrap()
}

/// A confirmed reservation of `quantity` for `[from, to)` hours.
pub async fn hold(
    repo: &ResourceRepo,
    res: &Resource,
    quantity: u32,
    from: i64,
    to: i64,
) -> Reservation {
    let r = ask(repo, res, quantity, from, to).await;
    let r = repo.reservations().confirm(r.id).await.unwrap();
    assert_eq!(r.state, ReservationState::Confirmed);
    r
}

/// Who is in which group, fixed for a test.
#[derive(Default)]
pub struct Roster(pub BTreeMap<ObjectId, Vec<String>>);

#[async_trait]
impl GroupLookup for Roster {
    async fn members(&self, group: ObjectId) -> Result<Vec<String>> {
        Ok(self.0.get(&group).cloned().unwrap_or_default())
    }
}
