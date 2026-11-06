//! Allocations: asked by one person, answered by the other, counted once accepted.

mod common;

use common::{cap, person, repo_for, setup, t};
use fs_resources_core::{AllocationState, Error};

const WEEK: i64 = 168;

#[tokio::test]
async fn only_an_accepted_allocation_counts_and_the_answer_can_be_taken_back() {
    let (_, store) = setup().await;
    let (a, b) = (
        repo_for(&store, &person("a")),
        repo_for(&store, &person("b")),
    );
    a.capacity().set(cap(600, 0, WEEK)).await.unwrap();
    let ask = b
        .capacity()
        .propose(&person("a"), "task", t(0), t(WEEK), 240)
        .await
        .unwrap();
    let load = || async {
        a.capacity()
            .load(t(0), t(WEEK))
            .await
            .unwrap()
            .allocated_minutes
    };
    assert_eq!(load().await, 0, "proposed does not count");
    a.capacity().answer(ask.id, true).await.unwrap();
    assert_eq!(load().await, 240);
    a.capacity().answer(ask.id, true).await.unwrap(); // repeating is fine
    assert_eq!(load().await, 240);
    let back = a.capacity().answer(ask.id, false).await.unwrap();
    assert_eq!(back.state, AllocationState::Declined);
    assert_eq!(load().await, 0, "the load follows the answer");
    a.capacity().answer(ask.id, true).await.unwrap();
    assert_eq!(load().await, 240);
}

#[tokio::test]
async fn the_proposer_withdraws_and_the_load_follows() {
    let (_, store) = setup().await;
    let (a, b) = (
        repo_for(&store, &person("a")),
        repo_for(&store, &person("b")),
    );
    let ask = b
        .capacity()
        .propose(&person("a"), "task", t(0), t(WEEK), 240)
        .await
        .unwrap();
    a.capacity().answer(ask.id, true).await.unwrap();
    let w = b.capacity().withdraw(ask.id).await.unwrap();
    assert_eq!(w.state, AllocationState::Withdrawn);
    assert_eq!(
        a.capacity()
            .load(t(0), t(WEEK))
            .await
            .unwrap()
            .allocated_minutes,
        0
    );
    // a withdrawn allocation cannot be answered or withdrawn again
    let err = a.capacity().answer(ask.id, true).await.unwrap_err();
    assert!(matches!(
        err,
        Error::InvalidTransition {
            from: "withdrawn",
            ..
        }
    ));
    let err = b.capacity().withdraw(ask.id).await.unwrap_err();
    assert!(matches!(
        err,
        Error::InvalidTransition {
            from: "withdrawn",
            ..
        }
    ));
    // the person still sees it, as withdrawn
    let seen = a.capacity().allocations(t(0), t(WEEK)).await.unwrap();
    assert_eq!(seen[0].state, AllocationState::Withdrawn);
}

#[tokio::test]
async fn a_declined_allocation_cannot_be_withdrawn() {
    let (_, store) = setup().await;
    let (a, b) = (
        repo_for(&store, &person("a")),
        repo_for(&store, &person("b")),
    );
    let ask = b
        .capacity()
        .propose(&person("a"), "task", t(0), t(WEEK), 60)
        .await
        .unwrap();
    a.capacity().answer(ask.id, false).await.unwrap();
    let err = b.capacity().withdraw(ask.id).await.unwrap_err();
    assert!(matches!(
        err,
        Error::InvalidTransition {
            from: "declined",
            ..
        }
    ));
}

#[tokio::test]
async fn allocations_are_listed_by_the_range_they_touch() {
    let (_, store) = setup().await;
    let (a, b) = (
        repo_for(&store, &person("a")),
        repo_for(&store, &person("b")),
    );
    b.capacity()
        .propose(&person("a"), "t1", t(0), t(24), 60)
        .await
        .unwrap();
    b.capacity()
        .propose(&person("a"), "t2", t(24), t(48), 60)
        .await
        .unwrap();
    let day2 = a.capacity().allocations(t(24), t(48)).await.unwrap();
    assert_eq!(
        day2.iter().map(|x| x.subject.as_str()).collect::<Vec<_>>(),
        ["t2"]
    );
    assert_eq!(
        a.capacity().allocations(t(0), t(48)).await.unwrap().len(),
        2
    );
}
