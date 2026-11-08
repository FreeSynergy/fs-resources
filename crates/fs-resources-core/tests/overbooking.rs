//! Two reservations may never hold more than exists, however they arrive.

mod common;

use common::{ask, hold, make, setup, t};
use fs_resources_core::{Error, ReservationState, ResourceKind};

#[tokio::test]
async fn two_confirms_that_together_exceed_the_quantity_run_concurrently_and_one_wins() {
    let (repo, _) = setup().await;
    let tent = make(&repo, ResourceKind::Thing, 5).await;
    let a = ask(&repo, &tent, 3, 0, 24).await;
    let b = ask(&repo, &tent, 3, 12, 36).await;
    let (c1, c2) = (repo.clone(), repo.clone());
    let (ra, rb) = tokio::join!(
        c1.reservations().confirm(a.id),
        c2.reservations().confirm(b.id)
    );
    assert_eq!(
        u8::from(ra.is_ok()) + u8::from(rb.is_ok()),
        1,
        "{ra:?} {rb:?}"
    );
    let refused = ra.err().or(rb.err()).unwrap();
    assert!(matches!(refused, Error::NotFree { wanted: 3, free: 2 }));
    let free = repo
        .reservations()
        .free(tent.id, t(0), t(36))
        .await
        .unwrap();
    assert_eq!(free.iter().map(|p| p.free).min(), Some(2));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn many_concurrent_confirms_never_hold_more_than_exists() {
    let (repo, _) = setup().await;
    let tent = make(&repo, ResourceKind::Thing, 4).await;
    let mut asked = vec![];
    for _ in 0..8 {
        asked.push(ask(&repo, &tent, 1, 0, 10).await);
    }
    let clones: Vec<_> = asked.iter().map(|_| repo.clone()).collect();
    let results = confirm_all(&clones, &asked).await;
    assert_eq!(results.iter().filter(|ok| **ok).count(), 4);
}

async fn confirm_all(
    repos: &[fs_resources_core::ResourceRepo],
    asked: &[fs_resources_core::Reservation],
) -> Vec<bool> {
    let mut tasks = vec![];
    for (r, a) in repos.iter().zip(asked) {
        let (r, id) = (r.clone(), a.id);
        tasks.push(tokio::spawn(async move {
            r.reservations().confirm(id).await.is_ok()
        }));
    }
    let mut out = vec![];
    for t in tasks {
        out.push(t.await.unwrap());
    }
    out
}

#[tokio::test]
async fn touching_ranges_do_not_conflict() {
    let (repo, _) = setup().await;
    let room = make(&repo, ResourceKind::Room, 1).await;
    hold(&repo, &room, 1, 0, 10).await;
    hold(&repo, &room, 1, 10, 20).await; // [0,10) and [10,20)
    let overlapping = ask(&repo, &room, 1, 9, 11).await;
    let err = repo
        .reservations()
        .confirm(overlapping.id)
        .await
        .unwrap_err();
    assert!(matches!(err, Error::NotFree { wanted: 1, free: 0 }));
}

#[tokio::test]
async fn a_request_holds_nothing() {
    let (repo, _) = setup().await;
    let room = make(&repo, ResourceKind::Room, 1).await;
    let first = ask(&repo, &room, 1, 0, 10).await;
    hold(&repo, &room, 1, 0, 10).await;
    let free = repo
        .reservations()
        .free(room.id, t(0), t(10))
        .await
        .unwrap();
    assert_eq!(free[0].free, 0);
    assert!(repo.reservations().confirm(first.id).await.is_err());
    let after = repo.reservations().get(first.id).await.unwrap().unwrap();
    assert_eq!(after.state, ReservationState::Requested);
}

#[tokio::test]
async fn change_that_does_not_fit_changes_nothing() {
    let (repo, _) = setup().await;
    let tent = make(&repo, ResourceKind::Thing, 5).await;
    let mine = hold(&repo, &tent, 3, 0, 10).await;
    hold(&repo, &tent, 2, 0, 10).await;
    let err = repo
        .reservations()
        .change(mine.id, 4, t(0), t(10))
        .await
        .unwrap_err();
    assert!(matches!(err, Error::NotFree { .. }));
    let same = repo.reservations().get(mine.id).await.unwrap().unwrap();
    assert_eq!(same, mine);
    let free = repo
        .reservations()
        .free(tent.id, t(0), t(10))
        .await
        .unwrap();
    assert_eq!((free.len(), free[0].free), (1, 0));
}

#[tokio::test]
async fn change_that_fits_keeps_the_hold_in_one_step() {
    let (repo, _) = setup().await;
    let tent = make(&repo, ResourceKind::Thing, 5).await;
    let mine = hold(&repo, &tent, 3, 0, 10).await;
    hold(&repo, &tent, 2, 0, 10).await;
    // growing into time that is free, and shrinking the quantity, both work; its own old hold does
    // not count against the new one
    let moved = repo
        .reservations()
        .change(mine.id, 3, t(5), t(20))
        .await
        .unwrap();
    assert_eq!(moved.state, ReservationState::Confirmed);
    assert_eq!((moved.from, moved.to), (t(5), t(20)));
    let free = repo
        .reservations()
        .free(tent.id, t(0), t(20))
        .await
        .unwrap();
    let at = |h: i64| {
        free.iter()
            .find(|p| p.from <= t(h) && t(h) < p.to)
            .unwrap()
            .free
    };
    assert_eq!((at(1), at(7), at(15)), (3, 0, 2));
}

#[tokio::test]
async fn set_quantity_is_refused_for_a_wrong_expected_value() {
    let (repo, _) = setup().await;
    let tent = make(&repo, ResourceKind::Thing, 5).await;
    let err = repo.items().set_quantity(tent.id, 4, 9).await.unwrap_err();
    assert!(matches!(err, Error::Conflict(_)));
    assert_eq!(
        repo.items().get(tent.id).await.unwrap().unwrap().quantity,
        5
    );
    let ok = repo.items().set_quantity(tent.id, 5, 9).await.unwrap();
    assert_eq!(ok.quantity, 9);
}

#[tokio::test]
async fn set_quantity_below_what_is_held_is_refused() {
    let (repo, _) = setup().await;
    let tent = make(&repo, ResourceKind::Thing, 5).await;
    hold(&repo, &tent, 3, 0, 10).await;
    hold(&repo, &tent, 2, 5, 15).await; // 5 are held between hour 5 and 10
    let err = repo.items().set_quantity(tent.id, 5, 4).await.unwrap_err();
    assert!(matches!(err, Error::Conflict(_)));
    assert_eq!(
        repo.items().get(tent.id).await.unwrap().unwrap().quantity,
        5
    );
    assert_eq!(
        repo.items()
            .set_quantity(tent.id, 5, 5)
            .await
            .unwrap()
            .quantity,
        5
    );
}
