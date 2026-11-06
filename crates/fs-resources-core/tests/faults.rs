//! A write that fails half way leaves a state that is safe, and a repeat finishes the job.

mod common;

use common::failing::Failing;
use common::{ME, Stores, hold, make, repo_for, stores, t};
use fs_resources_core::{NeedState, OfferState, ReservationState, ResourceKind, ResourceRepo};

/// A plain repo, and one over the same data whose main store can be made to fail.
async fn pair() -> (ResourceRepo, ResourceRepo, std::sync::Arc<Failing>, Stores) {
    let s = stores().await;
    let plain = repo_for(&s, ME);
    let f = Failing::new(s.main.clone());
    let faulty = ResourceRepo::new(f.clone(), s.private.clone(), ME);
    (plain, faulty, f, s)
}

#[tokio::test]
async fn a_fault_during_delete_never_raises_the_stock() {
    for fault_at in [1, 2, 3] {
        let (plain, faulty, f, _) = pair().await;
        let p = make(&plain, ResourceKind::Material, 10).await;
        for _ in 0..2 {
            let r = hold(&plain, &p, 3, 0, 24).await;
            plain.reservations().finish(r.id, 3).await.unwrap();
        }
        let stock = || async { plain.items().get(p.id).await.unwrap().map(|r| r.quantity) };
        assert_eq!(stock().await, Some(4));
        f.fail_nth_write(fault_at);
        assert!(faulty.items().delete(p.id).await.is_err());
        // the resource is erased first: it is either still there at 4, or gone, never 7
        assert!(
            matches!(stock().await, Some(4) | None),
            "fault at {fault_at}"
        );
        plain.items().delete(p.id).await.ok();
    }
}

#[tokio::test]
async fn a_fault_while_withdrawing_never_leaves_an_accepted_offer_without_its_hold() {
    let (plain, faulty, f, _) = pair().await;
    let tent = make(&plain, ResourceKind::Thing, 1).await;
    let n = plain
        .needs()
        .create("x", 1, "", Some(t(0)), Some(t(24)), "")
        .await
        .unwrap();
    let o = plain
        .needs()
        .offer(n.id, ME, 1, Some(tent.id), "")
        .await
        .unwrap();
    let o = plain.needs().decide(o.id, true).await.unwrap();
    let res = o.reservation.unwrap();

    f.fail_nth_write(2); // the offer is saved first; the release is the write that fails
    assert!(faulty.needs().withdraw(o.id).await.is_err());
    let offers = plain.needs().offers(n.id).await.unwrap();
    assert_eq!(offers[0].state, OfferState::Withdrawn);
    // the hold is still there (safe side), and the need is not told too early
    let r = plain.reservations().get(res).await.unwrap().unwrap();
    assert_eq!(r.state, ReservationState::Confirmed);
    // repeating the withdrawal finishes the job
    plain.needs().withdraw(o.id).await.unwrap();
    let r = plain.reservations().get(res).await.unwrap().unwrap();
    assert_eq!(r.state, ReservationState::Cancelled);
    assert_eq!(
        plain.needs().get(n.id).await.unwrap().unwrap().state,
        NeedState::Open
    );
}

#[tokio::test]
async fn a_fault_while_settling_is_healed_by_deciding_again() {
    let (plain, faulty, f, _) = pair().await;
    let n = plain
        .needs()
        .create("y", 1, "", None, None, "")
        .await
        .unwrap();
    let o = plain.needs().offer(n.id, ME, 1, None, "").await.unwrap();
    f.fail_nth_write(2); // the offer is saved, the need is not
    assert!(faulty.needs().decide(o.id, true).await.is_err());
    let need = plain.needs().get(n.id).await.unwrap().unwrap();
    assert_eq!((need.state, need.covered), (NeedState::Open, 0));
    let again = plain.needs().decide(o.id, true).await.unwrap();
    assert_eq!(again.state, OfferState::Accepted);
    let need = plain.needs().get(n.id).await.unwrap().unwrap();
    assert_eq!((need.state, need.covered), (NeedState::Covered, 1));
}
