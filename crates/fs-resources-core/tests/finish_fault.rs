//! Finishing material is one write, so a fault leaves nothing half done.

mod common;

use common::failing::Failing;
use common::{ME, hold, make, stores};
use fs_resources_core::{ReservationState, ResourceKind, ResourceRepo};

#[tokio::test]
async fn a_fault_on_the_only_write_changes_nothing_and_a_retry_works() {
    let s = stores().await;
    let plain = common::repo_for(&s, ME);
    let failing = Failing::new(s.main.clone());
    let faulty = ResourceRepo::new(failing.clone(), s.private.clone(), ME);
    let paint = make(&plain, ResourceKind::Material, 10).await;
    let r = hold(&plain, &paint, 6, 0, 24).await;

    failing.fail_nth_write(1);
    assert!(faulty.reservations().finish(r.id, 4).await.is_err());
    assert_eq!(
        plain.items().get(paint.id).await.unwrap().unwrap().quantity,
        10
    );
    let kept = plain.reservations().get(r.id).await.unwrap().unwrap();
    assert_eq!((kept.state, kept.done), (ReservationState::Confirmed, 0));

    // there is no second write that could fail: count the writes of a finish that goes through
    failing.fail_nth_write(2);
    faulty.reservations().finish(r.id, 4).await.unwrap();
    assert_eq!(
        plain.items().get(paint.id).await.unwrap().unwrap().quantity,
        6
    );
    let again = plain.reservations().finish(r.id, 4).await;
    assert!(again.is_err());
    assert_eq!(
        plain.items().get(paint.id).await.unwrap().unwrap().quantity,
        6
    );
}
