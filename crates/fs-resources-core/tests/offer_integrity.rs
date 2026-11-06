//! An accepted offer, its reservation and its need cannot drift apart.

mod common;

use common::{ME, make, setup, t};
use fs_resources_core::{Error, NeedState, ReservationState, ResourceKind, ResourceRepo};

async fn accepted(repo: &ResourceRepo) -> (fs_resources_core::Need, fs_resources_core::Offer) {
    let tent = make(repo, ResourceKind::Thing, 1).await;
    let n = repo
        .needs()
        .create("x", 1, "", Some(t(0)), Some(t(24)), "")
        .await
        .unwrap();
    let o = repo
        .needs()
        .offer(n.id, ME, 1, Some(tent.id), "")
        .await
        .unwrap();
    (n, repo.needs().decide(o.id, true).await.unwrap())
}

#[tokio::test]
async fn the_reservation_of_an_accepted_offer_cannot_be_cancelled_directly() {
    let (repo, _) = setup().await;
    let (n, o) = accepted(&repo).await;
    let id = o.reservation.unwrap();
    let err = repo.reservations().cancel(id).await.unwrap_err();
    assert!(matches!(err, Error::Conflict(_)));
    let r = repo.reservations().get(id).await.unwrap().unwrap();
    assert_eq!(r.state, ReservationState::Confirmed);
    let need = repo.needs().get(n.id).await.unwrap().unwrap();
    assert_eq!((need.state, need.covered), (NeedState::Covered, 1));
    // withdrawing the offer is the way, and then the reservation is gone
    repo.needs().withdraw(o.id).await.unwrap();
    assert_eq!(
        repo.reservations().get(id).await.unwrap().unwrap().state,
        ReservationState::Cancelled
    );
}

#[tokio::test]
async fn a_cancel_of_other_reservations_still_works() {
    let (repo, _) = setup().await;
    let tent = make(&repo, ResourceKind::Thing, 1).await;
    let r = common::hold(&repo, &tent, 1, 0, 5).await;
    repo.reservations().cancel(r.id).await.unwrap();
}

#[tokio::test]
async fn the_time_of_a_need_cannot_change_while_an_offer_is_accepted() {
    let (repo, _) = setup().await;
    let (n, o) = accepted(&repo).await;
    let err = repo
        .needs()
        .edit(n.id, "x", 1, "", Some(t(100)), Some(t(124)))
        .await
        .unwrap_err();
    assert!(matches!(err, Error::Conflict(_)));
    let same = repo.needs().get(n.id).await.unwrap().unwrap();
    assert_eq!((same.from, same.to), (Some(t(0)), Some(t(24))));
    // the same time, with another text, is fine
    repo.needs()
        .edit(n.id, "y", 1, "", Some(t(0)), Some(t(24)))
        .await
        .unwrap();
    // after the withdrawal the time is free to change
    repo.needs().withdraw(o.id).await.unwrap();
    let moved = repo
        .needs()
        .edit(n.id, "y", 1, "", Some(t(100)), Some(t(124)))
        .await
        .unwrap();
    assert_eq!(moved.from, Some(t(100)));
}

#[tokio::test]
async fn accepted_offers_of_the_largest_quantity_do_not_wrap_or_panic() {
    let (repo, _) = setup().await;
    let n = repo
        .needs()
        .create("x", 5, "", None, None, "")
        .await
        .unwrap();
    let o1 = repo
        .needs()
        .offer(n.id, ME, u32::MAX, None, "")
        .await
        .unwrap();
    let o2 = repo.needs().offer(n.id, ME, 1, None, "").await.unwrap();
    repo.needs().decide(o1.id, true).await.unwrap();
    repo.needs().decide(o2.id, true).await.unwrap();
    let need = repo.needs().get(n.id).await.unwrap().unwrap();
    assert_eq!((need.state, need.covered), (NeedState::Covered, u32::MAX));
    repo.needs().withdraw(o1.id).await.unwrap();
    let need = repo.needs().get(n.id).await.unwrap().unwrap();
    assert_eq!((need.state, need.covered), (NeedState::Open, 1));
}
