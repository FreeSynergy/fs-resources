//! A person's capacity belongs to that person: nobody else reads it, and nothing leaves the store.

mod common;

use common::{cap, person, repo_for, setup, t};
use fs_resources_core::{AllocationState, Error, Load, WRITABLE_TYPES};
use fs_store::traits::UnloggedObjects;
use fs_store::{ActivityFilter, ObjectId, Store};

const WEEK: i64 = 168;

fn same(a: &Error, b: &Error) {
    assert!(matches!(a, Error::Unknown(_)), "{a:?}");
    assert_eq!(format!("{a:?}"), format!("{b:?}"));
    assert_eq!(a.to_string(), b.to_string());
}

#[tokio::test]
async fn another_persons_record_answers_exactly_like_an_unknown_id() {
    let (_, store) = setup().await;
    let (a, b, c) = (
        repo_for(&store, &person("a")),
        repo_for(&store, &person("b")),
        repo_for(&store, &person("c")),
    );
    let mine = a.capacity().set(cap(600, 0, WEEK)).await.unwrap();
    let ask = c
        .capacity()
        .propose(&person("a"), "task", t(0), t(WEEK), 120)
        .await
        .unwrap();
    let unknown = ObjectId::new();

    same(
        &b.capacity().remove(mine.id).await.unwrap_err(),
        &b.capacity().remove(unknown).await.unwrap_err(),
    );
    same(
        &b.capacity().answer(ask.id, true).await.unwrap_err(),
        &b.capacity().answer(unknown, true).await.unwrap_err(),
    );
    same(
        &b.capacity().withdraw(ask.id).await.unwrap_err(),
        &b.capacity().withdraw(unknown).await.unwrap_err(),
    );
    // the person is not the proposer, the proposer is not the person
    same(
        &a.capacity().withdraw(ask.id).await.unwrap_err(),
        &a.capacity().withdraw(unknown).await.unwrap_err(),
    );
    same(
        &c.capacity().answer(ask.id, true).await.unwrap_err(),
        &c.capacity().answer(unknown, true).await.unwrap_err(),
    );
    // a capacity id is no allocation id and the other way round
    same(
        &a.capacity().answer(mine.id, true).await.unwrap_err(),
        &a.capacity().answer(unknown, true).await.unwrap_err(),
    );
    // nothing of it changed
    assert_eq!(a.capacity().mine(t(0), t(WEEK)).await.unwrap().len(), 1);
    let asked = a.capacity().allocations(t(0), t(WEEK)).await.unwrap();
    assert_eq!(asked[0].state, AllocationState::Proposed);
}

#[tokio::test]
async fn setting_with_an_id_of_someone_else_never_overwrites_it() {
    let (_, store) = setup().await;
    let (a, b) = (
        repo_for(&store, &person("a")),
        repo_for(&store, &person("b")),
    );
    let mine = a.capacity().set(cap(600, 0, WEEK)).await.unwrap();
    let mut theirs = cap(60, 0, WEEK);
    theirs.id = mine.id;
    let saved = b.capacity().set(theirs).await.unwrap();
    assert_ne!(saved.id, mine.id);
    assert_eq!(saved.person, person("b"));
    let kept = a.capacity().mine(t(0), t(WEEK)).await.unwrap();
    assert_eq!(
        (kept[0].minutes_per_week, kept[0].person.clone()),
        (600, person("a"))
    );
}

#[tokio::test]
async fn others_never_see_my_capacity_allocations_or_load() {
    let (_, store) = setup().await;
    let (a, b) = (
        repo_for(&store, &person("a")),
        repo_for(&store, &person("b")),
    );
    a.capacity().set(cap(600, 0, WEEK)).await.unwrap();
    let ask = b
        .capacity()
        .propose(&person("a"), "task", t(0), t(WEEK), 120)
        .await
        .unwrap();
    a.capacity().answer(ask.id, true).await.unwrap();
    assert_eq!(
        a.capacity()
            .load(t(0), t(WEEK))
            .await
            .unwrap()
            .capacity_minutes,
        600
    );

    assert!(b.capacity().mine(t(0), t(WEEK)).await.unwrap().is_empty());
    assert!(
        b.capacity()
            .allocations(t(0), t(WEEK))
            .await
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        b.capacity().load(t(0), t(WEEK)).await.unwrap(),
        Load::default()
    );
    assert!(b.capacity().shared_groups().await.unwrap().is_empty());
    // what b asked for is b's to see, with the answer
    let seen = b.capacity().proposed(t(0), t(WEEK)).await.unwrap();
    assert_eq!((seen.len(), seen[0].state), (1, AllocationState::Accepted));
    assert!(
        a.capacity()
            .proposed(t(0), t(WEEK))
            .await
            .unwrap()
            .is_empty()
    );
}

#[tokio::test]
async fn proposing_for_a_person_who_does_not_exist_answers_proposed() {
    let (_, store) = setup().await;
    let (a, b) = (
        repo_for(&store, &person("a")),
        repo_for(&store, &person("b")),
    );
    a.capacity().set(cap(600, 0, WEEK)).await.unwrap(); // a exists, "ghost" never did
    let known = b
        .capacity()
        .propose(&person("a"), "t", t(0), t(WEEK), 60)
        .await
        .unwrap();
    let ghost = b
        .capacity()
        .propose(&person("ghost"), "t", t(0), t(WEEK), 60)
        .await
        .unwrap();
    assert_eq!(known.state, AllocationState::Proposed);
    assert_eq!(ghost.state, AllocationState::Proposed);
    assert_eq!(ghost.proposer, person("b"));
}

#[tokio::test]
async fn nothing_private_is_in_the_activity_log_and_the_times_are_fixed() {
    let (repo, store) = setup().await;
    let (a, b) = (
        repo_for(&store, &person("a")),
        repo_for(&store, &person("b")),
    );
    let c = a.capacity().set(cap(600, 0, WEEK)).await.unwrap();
    let ask = b
        .capacity()
        .propose(&person("a"), "task", t(0), t(WEEK), 60)
        .await
        .unwrap();
    a.capacity().answer(ask.id, true).await.unwrap();
    a.capacity().answer(ask.id, false).await.unwrap();
    let group = ObjectId::new();
    a.capacity().share_with_group(group, true).await.unwrap();
    a.capacity().share_with_group(group, false).await.unwrap();
    b.capacity().withdraw(ask.id).await.unwrap_err();
    assert_eq!(store.log_stats().await.unwrap().entries, 0);

    // a public write does show up, and only that
    let r = common::make(&repo, fs_resources_core::ResourceKind::Thing, 1).await;
    let log = store.activities(&ActivityFilter::default()).await.unwrap();
    assert_eq!(log.iter().map(|x| x.object).collect::<Vec<_>>(), vec![r.id]);

    for id in [c.id, ask.id] {
        let o = store.read_unlogged(id).await.unwrap().unwrap();
        assert_eq!(o.envelope.published.timestamp(), 0);
        assert_eq!(o.envelope.updated.timestamp(), 0);
        assert!(o.envelope.attributed_to.is_none());
    }
}

#[test]
fn the_list_of_writable_types_does_not_name_private_ones() {
    for private in ["Capacity", "Allocation", "GroupShare"] {
        assert!(!WRITABLE_TYPES.contains(&private));
    }
    assert_eq!(WRITABLE_TYPES, ["Resource", "Reservation", "Need", "Offer"]);
}
