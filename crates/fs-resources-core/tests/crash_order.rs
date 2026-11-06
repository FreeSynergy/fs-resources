//! The order of the two writes that finish material, and what a crash between them leaves.

mod common;

use common::failing::Failing;
use common::{ME, hold, make, repo_for};
use fs_resources_core::{FINISH_MATERIAL_ORDER, ReservationState, ResourceKind, ResourceRepo};
use fs_store::TursoStore;
use std::sync::Arc;

/// A repository whose store can fail, a plain view of the same data, and 10 units of paint with a
/// confirmed reservation of 6.
async fn fixture() -> (
    ResourceRepo,
    Arc<Failing>,
    ResourceRepo,
    fs_resources_core::Resource,
    fs_resources_core::Reservation,
) {
    let store = Arc::new(TursoStore::open_in_memory().await.unwrap());
    let plain = repo_for(&store, ME);
    let failing = Failing::new(store);
    let faulty = ResourceRepo::new(failing.clone(), ME);
    let paint = make(&plain, ResourceKind::Material, 10).await;
    let r = hold(&plain, &paint, 6, 0, 24).await;
    (faulty, failing, plain, paint, r)
}

#[tokio::test]
async fn the_documented_order_is_stock_first_and_reservation_last() {
    assert_eq!(FINISH_MATERIAL_ORDER, ["resource", "reservation"]);
}

#[tokio::test]
async fn a_fault_on_the_first_write_leaves_everything_as_it_was() {
    let (faulty, failing, plain, paint, r) = fixture().await;
    failing.fail_nth_write(1);
    assert!(faulty.reservations().finish(r.id, 4).await.is_err());
    assert_eq!(
        plain.items().get(paint.id).await.unwrap().unwrap().quantity,
        10
    );
    let kept = plain.reservations().get(r.id).await.unwrap().unwrap();
    assert_eq!((kept.state, kept.done), (ReservationState::Confirmed, 0));
}

#[tokio::test]
async fn a_fault_on_the_second_write_leaves_a_low_stock_and_a_confirmed_reservation() {
    let (faulty, failing, plain, paint, r) = fixture().await;
    failing.fail_nth_write(2);
    assert!(faulty.reservations().finish(r.id, 4).await.is_err());
    // the stock is low, which is the safe side for a stock ...
    assert_eq!(
        plain.items().get(paint.id).await.unwrap().unwrap().quantity,
        6
    );
    // ... and the reservation did not move
    let kept = plain.reservations().get(r.id).await.unwrap().unwrap();
    assert_eq!((kept.state, kept.done), (ReservationState::Confirmed, 0));
    // the stated limit: repeating the finish lowers the stock a second time
    plain.reservations().finish(r.id, 4).await.unwrap();
    assert_eq!(
        plain.items().get(paint.id).await.unwrap().unwrap().quantity,
        2
    );
}
