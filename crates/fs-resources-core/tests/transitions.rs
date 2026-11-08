//! Every step the state does not allow is refused, and changes nothing.

mod common;

use common::{ME, ask, hold, make, setup, t};
use fs_objects::ObjectId;
use fs_resources_core::{Error, ReservationState, ResourceKind, ResourceRepo};

fn illegal<T: std::fmt::Debug>(r: Result<T, Error>, from: &str) {
    match r {
        Err(Error::InvalidTransition { from: f, .. }) => assert_eq!(f, from),
        other => panic!("expected an illegal step from {from}, got {other:?}"),
    }
}

async fn one_of_each(repo: &ResourceRepo) -> [ObjectId; 5] {
    let tent = make(repo, ResourceKind::Thing, 9).await;
    let requested = ask(repo, &tent, 1, 0, 1).await;
    let confirmed = hold(repo, &tent, 1, 0, 1).await;
    let declined = ask(repo, &tent, 1, 0, 1).await;
    repo.reservations().decline(declined.id).await.unwrap();
    let cancelled = ask(repo, &tent, 1, 0, 1).await;
    repo.reservations().cancel(cancelled.id).await.unwrap();
    let returned = hold(repo, &tent, 1, 0, 1).await;
    repo.reservations().finish(returned.id, 1).await.unwrap();
    [
        requested.id,
        confirmed.id,
        declined.id,
        cancelled.id,
        returned.id,
    ]
}

#[tokio::test]
async fn reservations_only_take_the_steps_of_their_state() {
    let (repo, _) = setup().await;
    let [requested, confirmed, declined, cancelled, returned] = one_of_each(&repo).await;
    let rs = repo.reservations();
    // confirm: only from requested
    illegal(rs.confirm(confirmed).await, "confirmed");
    illegal(rs.confirm(declined).await, "declined");
    illegal(rs.confirm(cancelled).await, "cancelled");
    illegal(rs.confirm(returned).await, "returned");
    // decline: only from requested
    illegal(rs.decline(confirmed).await, "confirmed");
    illegal(rs.decline(declined).await, "declined");
    illegal(rs.decline(cancelled).await, "cancelled");
    illegal(rs.decline(returned).await, "returned");
    // cancel: from requested or confirmed
    illegal(rs.cancel(declined).await, "declined");
    illegal(rs.cancel(cancelled).await, "cancelled");
    illegal(rs.cancel(returned).await, "returned");
    // finish: only from confirmed
    illegal(rs.finish(requested, 1).await, "requested");
    illegal(rs.finish(declined, 1).await, "declined");
    illegal(rs.finish(cancelled, 1).await, "cancelled");
    illegal(rs.finish(returned, 1).await, "returned");
    // change: only while requested or confirmed
    for (id, from) in [
        (declined, "declined"),
        (cancelled, "cancelled"),
        (returned, "returned"),
    ] {
        illegal(rs.change(id, 1, t(0), t(1)).await, from);
    }
    let states: Vec<_> = {
        let mut v = vec![];
        for id in [requested, confirmed, declined, cancelled, returned] {
            v.push(rs.get(id).await.unwrap().unwrap().state);
        }
        v
    };
    use ReservationState::{Cancelled, Confirmed, Declined, Requested, Returned};
    assert_eq!(
        states,
        [Requested, Confirmed, Declined, Cancelled, Returned]
    );
}

#[tokio::test]
async fn a_reservation_with_things_already_back_cannot_be_cancelled() {
    let (repo, _) = setup().await;
    let tent = make(&repo, ResourceKind::Thing, 5).await;
    let r = hold(&repo, &tent, 3, 0, 10).await;
    repo.reservations().finish(r.id, 1).await.unwrap();
    let err = repo.reservations().cancel(r.id).await.unwrap_err();
    assert!(matches!(err, Error::Conflict(_)));
}

#[tokio::test]
async fn needs_and_offers_only_take_the_steps_of_their_state() {
    let (repo, _) = setup().await;
    let needs = repo.needs();
    let need = needs.create("tents", 2, "", None, None, "").await.unwrap();
    needs.close(need.id).await.unwrap();
    illegal(needs.close(need.id).await, "closed");
    illegal(needs.edit(need.id, "x", 1, "", None, None).await, "closed");

    let open = needs.create("chairs", 5, "", None, None, "").await.unwrap();
    let accepted = needs.offer(open.id, ME, 1, None, "").await.unwrap();
    let accepted_offer = needs.decide(accepted.id, true).await.unwrap();
    // repeating the same decision is fine: it only settles again
    assert_eq!(
        needs.decide(accepted.id, true).await.unwrap(),
        accepted_offer
    );
    let declined = needs.offer(open.id, ME, 1, None, "").await.unwrap();
    needs.decide(declined.id, false).await.unwrap();
    illegal(needs.decide(declined.id, true).await, "declined");
    needs.decide(declined.id, false).await.unwrap(); // repeated: settles again
    illegal(needs.withdraw(declined.id).await, "declined");
    let withdrawn = needs.offer(open.id, ME, 1, None, "").await.unwrap();
    needs.withdraw(withdrawn.id).await.unwrap();
    needs.withdraw(withdrawn.id).await.unwrap(); // repeated: settles again
    illegal(needs.decide(withdrawn.id, true).await, "withdrawn");
    // the accepted one is still counted
    assert_eq!(needs.get(open.id).await.unwrap().unwrap().covered, 1);
}
