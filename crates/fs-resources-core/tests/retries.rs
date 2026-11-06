//! Retries after a fault never overwrite what happened in between, and listings stay whole.

mod common;

use common::failing::Failing;
use common::{ME, Stores, make, repo_for, stores, t};
use fs_resources_core::{OfferState, ReservationState, ResourceKind, ResourceRepo};

/// A plain repo, and one over the same data whose main store can be made to fail.
async fn pair() -> (ResourceRepo, ResourceRepo, std::sync::Arc<Failing>, Stores) {
    let s = stores().await;
    let plain = repo_for(&s, ME);
    let f = Failing::new(s.main.clone());
    let faulty = ResourceRepo::new(f.clone(), s.private.clone(), ME);
    (plain, faulty, f, s)
}

#[tokio::test]
async fn a_retried_accept_never_overwrites_a_reservation_that_exists() {
    let (plain, faulty, f, _) = pair().await;
    let paint = make(&plain, ResourceKind::Material, 10).await;
    let n = plain
        .needs()
        .create("x", 3, "", Some(t(0)), Some(t(24)), "")
        .await
        .unwrap();
    let o = plain
        .needs()
        .offer(n.id, ME, 3, Some(paint.id), "")
        .await
        .unwrap();
    f.fail_nth_write(3); // the reservation is saved, the accepted offer is not
    assert!(faulty.needs().decide(o.id, true).await.is_err());
    let stored = plain.needs().offers(n.id).await.unwrap().remove(0);
    let res = stored.reservation.unwrap();
    plain.reservations().finish(res, 3).await.unwrap();
    assert_eq!(
        plain.items().get(paint.id).await.unwrap().unwrap().quantity,
        7
    );

    let done = plain.needs().decide(o.id, true).await.unwrap();
    assert_eq!(
        (done.state, done.reservation),
        (OfferState::Accepted, Some(res))
    );
    let r = plain.reservations().get(res).await.unwrap().unwrap();
    assert_eq!((r.state, r.done), (ReservationState::Returned, 3));
    assert_eq!(
        plain.items().get(paint.id).await.unwrap().unwrap().quantity,
        7
    );
}

#[tokio::test]
async fn a_stuck_delete_marker_neither_repeats_a_row_nor_ends_a_page_early() {
    for size in [1, 2, 3] {
        let (plain, faulty, f, _) = pair().await;
        let mut ids = vec![];
        for name in ["a", "b", "c", "d", "e", "f"] {
            let r = plain
                .items()
                .create(ResourceKind::Thing, name, 1, "", "", "")
                .await
                .unwrap();
            ids.push(r.id);
        }
        for hide in [1, 2] {
            f.fail_nth_write(2); // the mark is written, the removal fails
            assert!(faulty.items().delete(ids[hide]).await.is_err());
        }
        let repo = plain.with_page_size(size);
        let (mut seen, mut cursor, mut pages) = (vec![], None, vec![]);
        loop {
            let p = repo
                .items()
                .list(&Default::default(), cursor.as_ref())
                .await
                .unwrap();
            pages.push(p.items.len());
            seen.extend(p.items.into_iter().map(|r| r.name));
            match p.next {
                Some(c) => cursor = Some(c),
                None => break,
            }
        }
        assert_eq!(seen, ["a", "d", "e", "f"], "page size {size}");
        // only the last page may be short
        assert!(
            pages[..pages.len() - 1].iter().all(|n| *n == size),
            "{pages:?}"
        );
    }
}
