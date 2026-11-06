//! Huge numbers, and listings that must agree with `free`.

mod common;

use common::{hold, make, setup, t};
use fs_resources_core::{Cursor, Error, ResourceFilter, ResourceKind};

#[tokio::test]
async fn a_huge_cursor_gives_an_empty_last_page() {
    let (repo, _) = setup().await;
    make(&repo, ResourceKind::Thing, 1).await;
    let c = Cursor(u64::MAX.to_string());
    let p = repo.needs().list("", false, Some(&c)).await.unwrap();
    assert!(p.items.is_empty() && p.next.is_none());
    let p = repo
        .items()
        .list(&ResourceFilter::default(), Some(&c))
        .await
        .unwrap();
    assert!(p.items.is_empty() && p.next.is_none());
    let tent = make(&repo, ResourceKind::Thing, 1).await;
    let p = repo
        .reservations()
        .of_resource(tent.id, t(0), t(1), Some(&c))
        .await
        .unwrap();
    assert!(p.items.is_empty());
    // a page near the top of the range does not overflow either
    let repo = repo.with_page_size(usize::MAX);
    repo.needs()
        .list("", false, Some(&Cursor(usize::MAX.to_string())))
        .await
        .unwrap();
}

#[tokio::test]
async fn set_quantity_that_would_wrap_the_base_is_refused() {
    let (repo, _) = setup().await;
    let p = make(&repo, ResourceKind::Material, 10).await;
    let x = hold(&repo, &p, 3, 0, 24).await;
    repo.reservations().finish(x.id, 3).await.unwrap();
    let err = repo
        .items()
        .set_quantity(p.id, 7, u32::MAX)
        .await
        .unwrap_err();
    assert!(matches!(err, Error::Invalid(_)));
    assert_eq!(repo.items().get(p.id).await.unwrap().unwrap().quantity, 7);
}

#[tokio::test]
async fn a_held_reservation_of_material_is_listed_after_its_to_as_free_counts_it() {
    let (repo, _) = setup().await;
    let paint = make(&repo, ResourceKind::Material, 10).await;
    let r = hold(&repo, &paint, 3, 0, 24).await;
    let listed = repo
        .reservations()
        .of_resource(paint.id, t(100), t(200), None)
        .await
        .unwrap();
    assert_eq!(
        listed.items.iter().map(|x| x.id).collect::<Vec<_>>(),
        [r.id]
    );
    let free = repo
        .reservations()
        .free(paint.id, t(100), t(200))
        .await
        .unwrap();
    assert_eq!(free[0].free, 7);
    // before it starts it is not part of the range
    assert!(
        repo.reservations()
            .of_resource(paint.id, t(-10), t(0), None)
            .await
            .unwrap()
            .items
            .is_empty()
    );
    // once used, it holds nothing and is listed only where it was
    repo.reservations().finish(r.id, 3).await.unwrap();
    assert!(
        repo.reservations()
            .of_resource(paint.id, t(100), t(200), None)
            .await
            .unwrap()
            .items
            .is_empty()
    );
    // a thing is listed by its own range only
    let tent = make(&repo, ResourceKind::Thing, 1).await;
    hold(&repo, &tent, 1, 0, 24).await;
    assert!(
        repo.reservations()
            .of_resource(tent.id, t(100), t(200), None)
            .await
            .unwrap()
            .items
            .is_empty()
    );
}
