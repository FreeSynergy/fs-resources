//! Two needs that want the same thing at the same time: one wins.

mod common;

use common::{ME, make, setup, t};
use fs_resources_core::{Error, ResourceKind};

#[tokio::test]
async fn two_needs_cannot_both_accept_the_same_single_tent_for_one_weekend() {
    let (repo, _) = setup().await;
    let tent = make(&repo, ResourceKind::Thing, 1).await;
    let (n1, n2) = (weekend(&repo).await, weekend(&repo).await);
    let o1 = repo
        .needs()
        .offer(n1.id, ME, 1, Some(tent.id), "")
        .await
        .unwrap();
    let o2 = repo
        .needs()
        .offer(n2.id, ME, 1, Some(tent.id), "")
        .await
        .unwrap();
    let (c1, c2) = (repo.clone(), repo.clone());
    let (a, b) = tokio::join!(
        c1.needs().decide(o1.id, true),
        c2.needs().decide(o2.id, true)
    );
    assert_eq!(a.is_ok() as u8 + b.is_ok() as u8, 1, "{a:?} {b:?}");
    let loser = a.err().or(b.err()).unwrap();
    assert!(matches!(loser, Error::NotFree { wanted: 1, free: 0 }));
    let held = repo
        .reservations()
        .of_resource(tent.id, t(0), t(300), None)
        .await
        .unwrap();
    assert_eq!(held.items.len(), 1);
}

async fn weekend(repo: &fs_resources_core::ResourceRepo) -> fs_resources_core::Need {
    repo.needs()
        .create("tent", 1, "piece", Some(t(120)), Some(t(168)), "")
        .await
        .unwrap()
}
