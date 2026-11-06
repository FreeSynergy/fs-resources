//! Material is used up; things come back.

mod common;

use common::{ask, hold, make, setup, t};
use fs_resources_core::{Error, ReservationState, ResourceKind};

#[tokio::test]
async fn monday_and_tuesday_reservations_of_the_whole_stock_cannot_both_be_confirmed() {
    let (repo, _) = setup().await;
    let paint = make(&repo, ResourceKind::Material, 10).await;
    let monday = ask(&repo, &paint, 10, 0, 24).await;
    let tuesday = ask(&repo, &paint, 10, 24, 48).await;
    repo.reservations().confirm(monday.id).await.unwrap();
    // Monday's hold has no end: what is used on Monday is gone on Tuesday.
    let err = repo.reservations().confirm(tuesday.id).await.unwrap_err();
    assert!(matches!(
        err,
        Error::NotFree {
            wanted: 10,
            free: 0
        }
    ));
    // and the other way round
    let (repo, _) = setup().await;
    let paint = make(&repo, ResourceKind::Material, 10).await;
    let tuesday = ask(&repo, &paint, 10, 24, 48).await;
    let monday = ask(&repo, &paint, 1, 0, 24).await;
    repo.reservations().confirm(tuesday.id).await.unwrap();
    assert!(repo.reservations().confirm(monday.id).await.is_err());
}

#[tokio::test]
async fn partial_use_lowers_the_stock_by_what_was_used_and_the_rest_returns() {
    let (repo, _) = setup().await;
    let paint = make(&repo, ResourceKind::Material, 10).await;
    let monday = hold(&repo, &paint, 10, 0, 24).await;
    let tuesday = ask(&repo, &paint, 6, 24, 48).await;
    let done = repo.reservations().finish(monday.id, 4).await.unwrap();
    assert_eq!((done.state, done.done), (ReservationState::Returned, 4));
    assert_eq!(
        repo.items().get(paint.id).await.unwrap().unwrap().quantity,
        6
    );
    // the six that were not used are free again, for exactly six
    repo.reservations().confirm(tuesday.id).await.unwrap();
    let seven = ask(&repo, &paint, 7, 48, 72).await;
    assert!(repo.reservations().confirm(seven.id).await.is_err());
}

#[tokio::test]
async fn a_second_finish_is_refused_and_does_not_lower_the_stock_again() {
    let (repo, _) = setup().await;
    let paint = make(&repo, ResourceKind::Material, 10).await;
    let r = hold(&repo, &paint, 5, 0, 24).await;
    repo.reservations().finish(r.id, 3).await.unwrap();
    let err = repo.reservations().finish(r.id, 3).await.unwrap_err();
    assert!(matches!(
        err,
        Error::InvalidTransition {
            from: "returned",
            ..
        }
    ));
    assert_eq!(
        repo.items().get(paint.id).await.unwrap().unwrap().quantity,
        7
    );
}

#[tokio::test]
async fn more_material_used_than_reserved_is_refused() {
    let (repo, _) = setup().await;
    let paint = make(&repo, ResourceKind::Material, 10).await;
    let r = hold(&repo, &paint, 5, 0, 24).await;
    let err = repo.reservations().finish(r.id, 6).await.unwrap_err();
    assert!(matches!(err, Error::Conflict(_)));
    assert_eq!(
        repo.items().get(paint.id).await.unwrap().unwrap().quantity,
        10
    );
    assert_eq!(
        repo.reservations().get(r.id).await.unwrap().unwrap().state,
        ReservationState::Confirmed
    );
}

#[tokio::test]
async fn partial_return_of_a_thing_frees_exactly_that_many() {
    let (repo, _) = setup().await;
    let tent = make(&repo, ResourceKind::Thing, 5).await;
    let r = hold(&repo, &tent, 4, 0, 24).await;
    let free_now = |repo: fs_resources_core::ResourceRepo, id| async move {
        repo.reservations().free(id, t(0), t(24)).await.unwrap()
    };
    assert_eq!(free_now(repo.clone(), tent.id).await[0].free, 1);
    let back = repo.reservations().finish(r.id, 1).await.unwrap();
    assert_eq!((back.state, back.done), (ReservationState::Confirmed, 1));
    assert_eq!(free_now(repo.clone(), tent.id).await[0].free, 2);
    let back = repo.reservations().finish(r.id, 2).await.unwrap();
    assert_eq!((back.state, back.done), (ReservationState::Confirmed, 3));
    assert_eq!(free_now(repo.clone(), tent.id).await[0].free, 4);
    let back = repo.reservations().finish(r.id, 1).await.unwrap();
    assert_eq!(back.state, ReservationState::Returned);
    assert_eq!(free_now(repo.clone(), tent.id).await[0].free, 5);
    // the stock of a thing never changes
    assert_eq!(
        repo.items().get(tent.id).await.unwrap().unwrap().quantity,
        5
    );
}

#[tokio::test]
async fn more_returned_than_is_out_is_refused() {
    let (repo, _) = setup().await;
    let tent = make(&repo, ResourceKind::Thing, 5).await;
    let r = hold(&repo, &tent, 3, 0, 24).await;
    repo.reservations().finish(r.id, 2).await.unwrap();
    let err = repo.reservations().finish(r.id, 2).await.unwrap_err();
    assert!(matches!(err, Error::Conflict(_)));
    let r = repo.reservations().get(r.id).await.unwrap().unwrap();
    assert_eq!(r.done, 2);
    repo.reservations().finish(r.id, 1).await.unwrap();
    let err = repo.reservations().finish(r.id, 1).await.unwrap_err();
    assert!(matches!(
        err,
        Error::InvalidTransition {
            from: "returned",
            ..
        }
    ));
}

#[tokio::test]
async fn a_room_follows_the_rules_of_a_thing() {
    let (repo, _) = setup().await;
    let hall = make(&repo, ResourceKind::Room, 1).await;
    let r = hold(&repo, &hall, 1, 0, 4).await;
    hold(&repo, &hall, 1, 4, 8).await;
    let r = repo.reservations().finish(r.id, 1).await.unwrap();
    assert_eq!(r.state, ReservationState::Returned);
    assert_eq!(
        repo.items().get(hall.id).await.unwrap().unwrap().quantity,
        1
    );
}
