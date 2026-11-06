//! The provider rule (ADR 0045): every provider has every method; what it does not offer says so.

mod common;

use std::sync::Arc;

use async_trait::async_trait;
use common::{Roster, setup};
use fs_interfaces::Capabilities;
use fs_resources_core::{
    Allocation, Capacities, Capacity, Cursor, Error, FreePeriod, Items, Load, Need, Needs, Offer,
    Page, Reservation, Reservations, Resource, ResourceFilter, ResourceKind, Result,
};
use fs_store::{ObjectId, Timestamp};

/// Every capability id named in `resources.capnp` (v0.20.0).
const CONTRACT: [&str; 18] = [
    "resource.create",
    "resource.edit",
    "resource.delete",
    "resource.read",
    "resource.subscribe",
    "reservation.request",
    "reservation.decide",
    "reservation.read",
    "capacity.set",
    "capacity.read",
    "capacity.share",
    "capacity.group",
    "allocation.propose",
    "allocation.answer",
    "need.create",
    "need.read",
    "offer.make",
    "offer.decide",
];

#[tokio::test]
async fn the_repo_reports_every_capability_of_the_contract() {
    let (repo, _) = setup().await;
    let repo = repo.with_groups(Arc::new(Roster::default()));
    let caps = repo.items().capabilities();
    for id in CONTRACT {
        assert!(caps.is_supported(id), "{id}");
    }
    assert_eq!(caps.entries().len(), CONTRACT.len());
}

#[tokio::test]
async fn without_the_groups_port_only_the_group_capability_says_no() {
    let (repo, _) = setup().await;
    let caps = repo.items().capabilities();
    for id in CONTRACT.iter().filter(|id| **id != "capacity.group") {
        assert!(caps.is_supported(id), "{id}");
    }
    let why = caps.why_not("capacity.group").unwrap();
    assert!(why.reason().contains("with_groups"));
}

/// A provider that can only read.
struct ReadOnly;

#[async_trait]
impl Items for ReadOnly {
    fn capabilities(&self) -> Capabilities {
        let mut c = Capabilities::new("read-only");
        c.supported("resource.read");
        c
    }
    async fn get(&self, _: ObjectId) -> Result<Option<Resource>> {
        Ok(None)
    }
    async fn list(&self, _: &ResourceFilter, _: Option<&Cursor>) -> Result<Page<Resource>> {
        Ok(Page {
            items: vec![],
            next: None,
        })
    }
}
#[async_trait]
impl Reservations for ReadOnly {
    async fn get(&self, _: ObjectId) -> Result<Option<Reservation>> {
        Ok(None)
    }
    async fn of_resource(
        &self,
        _: ObjectId,
        _: Timestamp,
        _: Timestamp,
        _: Option<&Cursor>,
    ) -> Result<Page<Reservation>> {
        Ok(Page {
            items: vec![],
            next: None,
        })
    }
    async fn of_holder(&self, _: &str, _: Option<&Cursor>) -> Result<Page<Reservation>> {
        Ok(Page {
            items: vec![],
            next: None,
        })
    }
    async fn free(&self, _: ObjectId, _: Timestamp, _: Timestamp) -> Result<Vec<FreePeriod>> {
        Ok(vec![])
    }
}
#[async_trait]
impl Capacities for ReadOnly {
    async fn mine(&self, _: Timestamp, _: Timestamp) -> Result<Vec<Capacity>> {
        Ok(vec![])
    }
    async fn allocations(&self, _: Timestamp, _: Timestamp) -> Result<Vec<Allocation>> {
        Ok(vec![])
    }
    async fn proposed(&self, _: Timestamp, _: Timestamp) -> Result<Vec<Allocation>> {
        Ok(vec![])
    }
    async fn load(&self, _: Timestamp, _: Timestamp) -> Result<Load> {
        Ok(Load::default())
    }
    async fn shared_groups(&self) -> Result<Vec<ObjectId>> {
        Ok(vec![])
    }
}
#[async_trait]
impl Needs for ReadOnly {
    async fn get(&self, _: ObjectId) -> Result<Option<Need>> {
        Ok(None)
    }
    async fn list(&self, _: &str, _: bool, _: Option<&Cursor>) -> Result<Page<Need>> {
        Ok(Page {
            items: vec![],
            next: None,
        })
    }
    async fn offers(&self, _: ObjectId) -> Result<Vec<Offer>> {
        Ok(vec![])
    }
}

fn says<T: std::fmt::Debug>(r: Result<T>, capability: &str) {
    match r {
        Err(Error::Unsupported(u)) => assert_eq!(u.capability(), capability),
        other => panic!("expected Unsupported({capability}), got {other:?}"),
    }
}

#[tokio::test]
async fn what_a_provider_does_not_override_says_unsupported() {
    let (id, t0) = (ObjectId::new(), common::t(0));
    let p = ReadOnly;
    says(
        Items::create(&p, ResourceKind::Thing, "x", 1, "", "", "").await,
        "resource.create",
    );
    says(
        Items::edit(&p, id, "x", "", "", "", &[]).await,
        "resource.edit",
    );
    says(Items::set_quantity(&p, id, 1, 2).await, "resource.edit");
    says(Items::delete(&p, id).await, "resource.delete");
    says(
        Reservations::request(&p, id, 1, t0, t0, "h", "", "").await,
        "reservation.request",
    );
    says(Reservations::confirm(&p, id).await, "reservation.decide");
    says(Reservations::decline(&p, id).await, "reservation.decide");
    says(Reservations::cancel(&p, id).await, "reservation.request");
    says(Reservations::finish(&p, id, 1).await, "reservation.decide");
    says(
        Reservations::change(&p, id, 1, t0, t0).await,
        "reservation.request",
    );
    says(Capacities::remove(&p, id).await, "capacity.set");
    says(
        Capacities::propose(&p, "x", "", t0, t0, 1).await,
        "allocation.propose",
    );
    says(Capacities::answer(&p, id, true).await, "allocation.answer");
    says(Capacities::withdraw(&p, id).await, "allocation.propose");
    says(
        Capacities::share_with_group(&p, id, true).await,
        "capacity.share",
    );
    says(
        Capacities::group_load(&p, id, t0, t0).await,
        "capacity.group",
    );
    says(
        Needs::create(&p, "x", 1, "", None, None, "").await,
        "need.create",
    );
    says(Needs::close(&p, id).await, "need.create");
    says(Needs::offer(&p, id, "x", 1, None, "").await, "offer.make");
    says(Needs::withdraw(&p, id).await, "offer.make");
    says(Needs::decide(&p, id, true).await, "offer.decide");
    assert!(Items::get(&p, id).await.unwrap().is_none());
}
