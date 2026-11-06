//! Offers: the reservation an accepted offer makes, and what undoes it.

mod common;

use common::{ME, hold, make, person, setup, t};
use fs_resources_core::{
    Error, Need, NeedState, OfferState, ReservationState, ResourceKind, ResourceRepo,
};

const PROJECT: &str = "https://example.org/projects/fair";

async fn weekend_need(repo: &ResourceRepo, quantity: u32) -> Need {
    repo.needs()
        .create(
            "tents",
            quantity,
            "piece",
            Some(t(120)),
            Some(t(168)),
            PROJECT,
        )
        .await
        .unwrap()
}

#[tokio::test]
async fn accepting_with_a_resource_and_a_time_makes_a_confirmed_reservation() {
    let (repo, _) = setup().await;
    let tent = make(&repo, ResourceKind::Thing, 3).await;
    let need = weekend_need(&repo, 2).await;
    let offer = repo
        .needs()
        .offer(need.id, &person("a"), 2, Some(tent.id), "ours")
        .await
        .unwrap();
    assert!(offer.reservation.is_none());
    let offer = repo.needs().decide(offer.id, true).await.unwrap();
    let r = repo
        .reservations()
        .get(offer.reservation.unwrap())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(r.state, ReservationState::Confirmed);
    assert_eq!(
        (r.resource, r.quantity, r.from, r.to),
        (tent.id, 2, t(120), t(168))
    );
    assert_eq!(r.holder, PROJECT);
    let free = repo
        .reservations()
        .free(tent.id, t(0), t(200))
        .await
        .unwrap();
    assert_eq!(free.iter().map(|p| p.free).collect::<Vec<_>>(), [3, 1, 3]);
    // the stored offer carries the id
    let stored = &repo.needs().offers(need.id).await.unwrap()[0];
    assert_eq!(stored.reservation, Some(r.id));
}

#[tokio::test]
async fn an_offer_without_a_resource_or_a_need_without_a_time_makes_no_reservation() {
    let (repo, _) = setup().await;
    let tent = make(&repo, ResourceKind::Thing, 3).await;
    let timed = weekend_need(&repo, 1).await;
    let plain = repo.needs().offer(timed.id, ME, 1, None, "").await.unwrap();
    assert!(
        repo.needs()
            .decide(plain.id, true)
            .await
            .unwrap()
            .reservation
            .is_none()
    );
    let untimed = repo
        .needs()
        .create("tents", 1, "", None, None, "")
        .await
        .unwrap();
    let o = repo
        .needs()
        .offer(untimed.id, ME, 1, Some(tent.id), "")
        .await
        .unwrap();
    assert!(
        repo.needs()
            .decide(o.id, true)
            .await
            .unwrap()
            .reservation
            .is_none()
    );
    assert!(
        repo.reservations()
            .of_holder(PROJECT, None)
            .await
            .unwrap()
            .items
            .is_empty()
    );
}

#[tokio::test]
async fn when_it_is_not_free_nothing_changes() {
    let (repo, _) = setup().await;
    let tent = make(&repo, ResourceKind::Thing, 3).await;
    hold(&repo, &tent, 2, 100, 200).await;
    let need = weekend_need(&repo, 2).await;
    let offer = repo
        .needs()
        .offer(need.id, ME, 2, Some(tent.id), "")
        .await
        .unwrap();
    let err = repo.needs().decide(offer.id, true).await.unwrap_err();
    assert!(matches!(err, Error::NotFree { wanted: 2, free: 1 }));
    let offers = repo.needs().offers(need.id).await.unwrap();
    assert_eq!(
        (offers[0].state, offers[0].reservation),
        (OfferState::Offered, None)
    );
    let need = repo.needs().get(need.id).await.unwrap().unwrap();
    assert_eq!((need.state, need.covered), (NeedState::Open, 0));
    assert!(
        repo.reservations()
            .of_holder(PROJECT, None)
            .await
            .unwrap()
            .items
            .is_empty()
    );
}

#[tokio::test]
async fn withdrawing_or_declining_an_accepted_offer_cancels_its_reservation() {
    let (repo, _) = setup().await;
    let tent = make(&repo, ResourceKind::Thing, 3).await;
    let need = weekend_need(&repo, 2).await;
    for withdraw in [true, false] {
        let o = repo
            .needs()
            .offer(need.id, ME, 2, Some(tent.id), "")
            .await
            .unwrap();
        let o = repo.needs().decide(o.id, true).await.unwrap();
        assert_eq!(
            repo.needs().get(need.id).await.unwrap().unwrap().state,
            NeedState::Covered
        );
        let ended = if withdraw {
            repo.needs().withdraw(o.id).await.unwrap()
        } else {
            repo.needs().decide(o.id, false).await.unwrap()
        };
        let r = repo
            .reservations()
            .get(o.reservation.unwrap())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(r.state, ReservationState::Cancelled);
        assert_eq!(
            ended.state,
            if withdraw {
                OfferState::Withdrawn
            } else {
                OfferState::Declined
            }
        );
        let free = repo
            .reservations()
            .free(tent.id, t(0), t(200))
            .await
            .unwrap();
        assert_eq!(free.len(), 1);
        assert_eq!(
            repo.needs().get(need.id).await.unwrap().unwrap().state,
            NeedState::Open
        );
    }
}
