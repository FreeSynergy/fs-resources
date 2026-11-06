//! `set` says nothing about ids of others, and refuses zero minutes.

mod common;

use common::{cap, person, repo_for, setup, t};
use fs_resources_core::Error;
use fs_store::ObjectId;

const WEEK: i64 = 168;

#[tokio::test]
async fn a_given_id_is_kept_only_when_the_record_is_mine() {
    let (_, s) = setup().await;
    let (a, b) = (repo_for(&s, &person("a")), repo_for(&s, &person("b")));
    let theirs = a.capacity().set(cap(600, 0, WEEK)).await.unwrap();
    let mut known = cap(1, 0, 1);
    known.id = theirs.id; // an id that exists, of someone else
    let mut unknown = cap(1, 2, 3);
    unknown.id = ObjectId::new(); // an id that exists nowhere
    let (r1, r2) = (
        b.capacity().set(known).await.unwrap(),
        b.capacity().set(unknown.clone()).await.unwrap(),
    );
    assert_ne!(r1.id, theirs.id);
    assert_ne!(r2.id, unknown.id, "both get a fresh id, so nothing is told");
    // a's record is untouched
    assert_eq!(
        a.capacity().mine(t(0), t(WEEK)).await.unwrap()[0].minutes_per_week,
        600
    );
    // my own id is kept, and the record replaced
    let mut mine = r1.clone();
    mine.minutes_per_week = 5;
    let kept = b.capacity().set(mine).await.unwrap();
    assert_eq!(kept.id, r1.id);
    assert_eq!(b.capacity().mine(t(0), t(WEEK)).await.unwrap().len(), 2);
}

#[tokio::test]
async fn zero_minutes_per_week_is_refused() {
    let (repo, _) = setup().await;
    let r = repo.capacity().set(cap(0, 0, WEEK)).await;
    assert!(matches!(r, Err(Error::Invalid(_))), "{r:?}");
}
