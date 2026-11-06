//! Needs: covering, closing, deleting, listing.

mod common;

use common::{ME, person, setup};
use fs_resources_core::{Error, Need, NeedState, OfferState};

const PROJECT: &str = "https://example.org/projects/fair";

#[tokio::test]
async fn covered_goes_up_with_accepted_offers_and_back_down() {
    let (repo, _) = setup().await;
    let needs = repo.needs();
    let need = needs
        .create("chairs", 3, "piece", None, None, PROJECT)
        .await
        .unwrap();
    let state = || async { needs.get(need.id).await.unwrap().unwrap() };
    let two = needs
        .offer(need.id, &person("a"), 2, None, "")
        .await
        .unwrap();
    let one = needs
        .offer(need.id, &person("b"), 1, None, "")
        .await
        .unwrap();
    assert_eq!(
        (state().await.state, state().await.covered),
        (NeedState::Open, 0)
    );
    needs.decide(two.id, true).await.unwrap();
    assert_eq!(
        (state().await.state, state().await.covered),
        (NeedState::Open, 2)
    );
    needs.decide(one.id, true).await.unwrap();
    assert_eq!(
        (state().await.state, state().await.covered),
        (NeedState::Covered, 3)
    );
    needs.withdraw(two.id).await.unwrap();
    assert_eq!(
        (state().await.state, state().await.covered),
        (NeedState::Open, 1)
    );
    needs.decide(one.id, false).await.unwrap();
    assert_eq!(
        (state().await.state, state().await.covered),
        (NeedState::Open, 0)
    );
    assert_eq!(
        needs
            .offers(need.id)
            .await
            .unwrap()
            .iter()
            .map(|o| o.state)
            .collect::<Vec<_>>(),
        [OfferState::Withdrawn, OfferState::Declined]
    );
}

#[tokio::test]
async fn lowering_the_quantity_can_cover_a_need() {
    let (repo, _) = setup().await;
    let needs = repo.needs();
    let need = needs.create("chairs", 3, "", None, None, "").await.unwrap();
    let o = needs.offer(need.id, ME, 2, None, "").await.unwrap();
    needs.decide(o.id, true).await.unwrap();
    let edited = needs
        .edit(need.id, "chairs", 2, "", None, None)
        .await
        .unwrap();
    assert_eq!(edited.state, NeedState::Covered);
    let edited = needs
        .edit(need.id, "chairs", 5, "", None, None)
        .await
        .unwrap();
    assert_eq!(edited.state, NeedState::Open);
}

#[tokio::test]
async fn a_closed_need_refuses_offers() {
    let (repo, _) = setup().await;
    let needs = repo.needs();
    let need = needs.create("chairs", 3, "", None, None, "").await.unwrap();
    let before = needs.offer(need.id, ME, 1, None, "").await.unwrap();
    needs.close(need.id).await.unwrap();
    let err = needs.offer(need.id, ME, 1, None, "").await.unwrap_err();
    assert!(matches!(err, Error::Conflict(_)));
    assert!(matches!(
        needs.decide(before.id, true).await,
        Err(Error::Conflict(_))
    ));
    assert_eq!(needs.offers(need.id).await.unwrap().len(), 1);
}

#[tokio::test]
async fn a_need_with_an_accepted_offer_cannot_be_deleted_and_otherwise_takes_its_offers_along() {
    let (repo, _) = setup().await;
    let needs = repo.needs();
    let need = needs.create("chairs", 3, "", None, None, "").await.unwrap();
    let o = needs.offer(need.id, ME, 1, None, "").await.unwrap();
    needs.decide(o.id, true).await.unwrap();
    assert!(matches!(
        needs.delete(need.id).await,
        Err(Error::Conflict(_))
    ));
    needs.withdraw(o.id).await.unwrap();
    needs.delete(need.id).await.unwrap();
    assert!(needs.get(need.id).await.unwrap().is_none());
    assert!(needs.offers(need.id).await.unwrap().is_empty());
}

#[tokio::test]
async fn needs_are_listed_by_subject_and_openness() {
    let (repo, _) = setup().await;
    let needs = repo.needs();
    let a = needs.create("a", 1, "", None, None, "s1").await.unwrap();
    needs.create("b", 1, "", None, None, "s1").await.unwrap();
    needs.create("c", 1, "", None, None, "s2").await.unwrap();
    needs.close(a.id).await.unwrap();
    let ids = |p: fs_resources_core::Page<Need>| {
        p.items.iter().map(|n| n.what.clone()).collect::<Vec<_>>()
    };
    assert_eq!(
        ids(needs.list("s1", false, None).await.unwrap()),
        ["a", "b"]
    );
    assert_eq!(ids(needs.list("s1", true, None).await.unwrap()), ["b"]);
    assert_eq!(needs.list("", false, None).await.unwrap().items.len(), 3);
}
