//! The feed carries resources, reservations, needs and offers, and never anything private.

mod common;

use std::sync::{Arc, Mutex};
use std::time::Duration;

use common::{ME, cap, make, person, repo_for, setup, t};
use fs_resources_core::{
    EntityKind, ResourceChange, ResourceEntity, ResourceFeed, ResourceKind, ResourceListener,
};
use fs_store::ObjectId;

#[derive(Default)]
struct Log(Mutex<Vec<ResourceChange>>);

impl ResourceListener for Log {
    fn changed(&self, change: &ResourceChange) {
        self.0.lock().unwrap().push(*change);
    }
}

async fn settle() {
    tokio::time::sleep(Duration::from_millis(100)).await;
}

fn entity(kind: EntityKind, id: ObjectId) -> ResourceEntity {
    ResourceEntity { kind, id }
}

#[tokio::test]
async fn public_changes_arrive_in_order() {
    let (repo, _) = setup().await;
    let log = Arc::new(Log::default());
    let _sub = repo.subscribe(log.clone());
    let tent = make(&repo, ResourceKind::Thing, 2).await;
    let r = common::hold(&repo, &tent, 1, 0, 5).await;
    let need = repo
        .needs()
        .create("chairs", 1, "", None, None, "")
        .await
        .unwrap();
    let offer = repo.needs().offer(need.id, ME, 1, None, "").await.unwrap();
    repo.reservations().cancel(r.id).await.unwrap();
    repo.items().delete(tent.id).await.unwrap();
    settle().await;
    use EntityKind::{Need, Offer, Reservation, Resource};
    use ResourceChange::{Created, Deleted, Updated};
    let got = log.0.lock().unwrap().clone();
    assert_eq!(
        got,
        vec![
            Created(entity(Resource, tent.id)),
            Created(entity(Reservation, r.id)), // requested
            Updated(entity(Reservation, r.id)), // confirmed
            Created(entity(Need, need.id)),
            Created(entity(Offer, offer.id)),
            Updated(entity(Reservation, r.id)), // cancelled
            Deleted(entity(Resource, tent.id)),
            Deleted(entity(Reservation, r.id)),
        ]
    );
}

#[tokio::test]
async fn capacity_allocations_and_shares_never_arrive() {
    let (repo, store) = setup().await;
    let log = Arc::new(Log::default());
    let _sub = repo.subscribe(log.clone());
    let (a, b) = (
        repo_for(&store, &person("a")),
        repo_for(&store, &person("b")),
    );
    let c = a.capacity().set(cap(600, 0, 168)).await.unwrap();
    let ask = b
        .capacity()
        .propose(&person("a"), "task", t(0), t(168), 60)
        .await
        .unwrap();
    a.capacity().answer(ask.id, true).await.unwrap();
    a.capacity()
        .share_with_group(ObjectId::new(), true)
        .await
        .unwrap();
    b.capacity().withdraw(ask.id).await.unwrap();
    a.capacity().remove(c.id).await.unwrap();
    settle().await;
    assert!(log.0.lock().unwrap().is_empty());
    // the feed is alive: a public change after all that still arrives
    let tent = make(&repo, ResourceKind::Thing, 1).await;
    settle().await;
    assert_eq!(
        *log.0.lock().unwrap(),
        vec![ResourceChange::Created(entity(
            EntityKind::Resource,
            tent.id
        ))]
    );
}

#[tokio::test]
async fn dropping_the_subscription_ends_it() {
    let (repo, _) = setup().await;
    let log = Arc::new(Log::default());
    let sub = repo.subscribe(log.clone());
    make(&repo, ResourceKind::Thing, 1).await;
    settle().await;
    assert_eq!(log.0.lock().unwrap().len(), 1);
    drop(sub);
    settle().await;
    make(&repo, ResourceKind::Thing, 1).await;
    settle().await;
    assert_eq!(log.0.lock().unwrap().len(), 1);
}
