//! Reservations listed by resource and by holder, with paging.
#![allow(clippy::assert_is_empty)] // tests: plain `is_empty()` asserts read clearer than typed empty comparisons

mod common;

use common::{ME, make, person, setup};
use fs_resources_core::ResourceKind;

#[tokio::test]
async fn reservations_list_by_resource_and_holder_with_paging() {
    let (repo, _) = setup().await;
    let repo = repo.with_page_size(2);
    let tent = make(&repo, ResourceKind::Thing, 5).await;
    for h in [0, 10, 20] {
        common::ask(&repo, &tent, 1, h, h + 5).await;
    }
    let rs = repo.reservations();
    let first = rs
        .of_resource(tent.id, common::t(0), common::t(100), None)
        .await
        .unwrap();
    assert_eq!(first.items.len(), 2);
    let rest = rs
        .of_resource(tent.id, common::t(0), common::t(100), first.next.as_ref())
        .await
        .unwrap();
    assert_eq!((rest.items.len(), rest.next), (1, None));
    let window = rs
        .of_resource(tent.id, common::t(5), common::t(10), None)
        .await
        .unwrap();
    assert!(
        window.items.is_empty(),
        "[0,5) and [10,15) only touch [5,10)"
    );
    assert_eq!(rs.of_holder(ME, None).await.unwrap().items.len(), 2);
    assert!(
        rs.of_holder(&person("x"), None)
            .await
            .unwrap()
            .items
            .is_empty()
    );
}
