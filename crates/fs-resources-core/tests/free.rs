//! `free`: stretches of equal free quantity.

mod common;

use common::{ask, hold, make, setup, t};
use fs_resources_core::{Error, FreePeriod, ResourceKind};

fn p(from: i64, to: i64, free: u32) -> FreePeriod {
    FreePeriod {
        from: t(from),
        to: t(to),
        free,
    }
}

#[tokio::test]
async fn nothing_held_is_one_stretch_of_everything() {
    let (repo, _) = setup().await;
    let tent = make(&repo, ResourceKind::Thing, 5).await;
    let got = repo
        .reservations()
        .free(tent.id, t(0), t(20))
        .await
        .unwrap();
    assert_eq!(got, vec![p(0, 20, 5)]);
}

#[tokio::test]
async fn a_hold_in_the_middle_makes_three_stretches() {
    let (repo, _) = setup().await;
    let tent = make(&repo, ResourceKind::Thing, 5).await;
    hold(&repo, &tent, 3, 10, 12).await;
    let got = repo
        .reservations()
        .free(tent.id, t(0), t(20))
        .await
        .unwrap();
    assert_eq!(got, vec![p(0, 10, 5), p(10, 12, 2), p(12, 20, 5)]);
}

#[tokio::test]
async fn neighbours_with_equal_free_quantity_are_merged() {
    let (repo, _) = setup().await;
    let tent = make(&repo, ResourceKind::Thing, 5).await;
    hold(&repo, &tent, 3, 10, 12).await;
    hold(&repo, &tent, 3, 12, 14).await;
    let got = repo
        .reservations()
        .free(tent.id, t(0), t(20))
        .await
        .unwrap();
    assert_eq!(got, vec![p(0, 10, 5), p(10, 14, 2), p(14, 20, 5)]);
    // overlapping holds that add up, then leave one by one
    hold(&repo, &tent, 1, 11, 13).await;
    let got = repo
        .reservations()
        .free(tent.id, t(0), t(20))
        .await
        .unwrap();
    assert_eq!(
        got,
        vec![
            p(0, 10, 5),
            p(10, 11, 2),
            p(11, 13, 1),
            p(13, 14, 2),
            p(14, 20, 5)
        ]
    );
}

#[tokio::test]
async fn the_range_cuts_the_stretches_at_its_edges() {
    let (repo, _) = setup().await;
    let tent = make(&repo, ResourceKind::Thing, 5).await;
    hold(&repo, &tent, 3, 10, 14).await;
    let got = repo
        .reservations()
        .free(tent.id, t(12), t(16))
        .await
        .unwrap();
    assert_eq!(got, vec![p(12, 14, 2), p(14, 16, 5)]);
    let inside = repo
        .reservations()
        .free(tent.id, t(11), t(13))
        .await
        .unwrap();
    assert_eq!(inside, vec![p(11, 13, 2)]);
}

#[tokio::test]
async fn a_hold_that_only_touches_the_range_does_not_count() {
    let (repo, _) = setup().await;
    let tent = make(&repo, ResourceKind::Thing, 5).await;
    hold(&repo, &tent, 3, 0, 10).await;
    hold(&repo, &tent, 4, 20, 30).await;
    let got = repo
        .reservations()
        .free(tent.id, t(10), t(20))
        .await
        .unwrap();
    assert_eq!(got, vec![p(10, 20, 5)]);
}

#[tokio::test]
async fn a_hold_of_material_has_no_end() {
    let (repo, _) = setup().await;
    let paint = make(&repo, ResourceKind::Material, 10).await;
    hold(&repo, &paint, 4, 10, 12).await; // its `to` is ignored
    let got = repo
        .reservations()
        .free(paint.id, t(0), t(1000))
        .await
        .unwrap();
    assert_eq!(got, vec![p(0, 10, 10), p(10, 1000, 6)]);
}

#[tokio::test]
async fn requests_and_cancelled_reservations_hold_nothing() {
    let (repo, _) = setup().await;
    let tent = make(&repo, ResourceKind::Thing, 5).await;
    ask(&repo, &tent, 3, 0, 10).await;
    let r = hold(&repo, &tent, 2, 0, 10).await;
    let got = repo
        .reservations()
        .free(tent.id, t(0), t(10))
        .await
        .unwrap();
    assert_eq!(got, vec![p(0, 10, 3)]);
    repo.reservations().cancel(r.id).await.unwrap();
    let got = repo
        .reservations()
        .free(tent.id, t(0), t(10))
        .await
        .unwrap();
    assert_eq!(got, vec![p(0, 10, 5)]);
}

#[tokio::test]
async fn an_empty_range_is_refused() {
    let (repo, _) = setup().await;
    let tent = make(&repo, ResourceKind::Thing, 5).await;
    let err = repo
        .reservations()
        .free(tent.id, t(5), t(5))
        .await
        .unwrap_err();
    assert!(matches!(err, Error::Invalid(_)));
}
