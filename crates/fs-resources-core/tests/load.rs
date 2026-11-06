//! Capacity ranges may not overlap; the load is pro rata, in whole minutes, rounded down.

mod common;

use common::{cap, person, repo_for, setup, t};
use fs_resources_core::{Error, Load};

const WEEK: i64 = 168;

#[tokio::test]
async fn overlapping_ranges_are_refused_and_touching_ones_are_not() {
    let (repo, _) = setup().await;
    let cs = repo.capacity();
    let first = cs.set(cap(600, 0, WEEK)).await.unwrap();
    let err = cs.set(cap(300, WEEK - 1, 2 * WEEK)).await.unwrap_err();
    assert!(matches!(err, Error::Conflict(_)));
    let err = cs.set(cap(300, -5, 1)).await.unwrap_err();
    assert!(matches!(err, Error::Conflict(_)));
    cs.set(cap(300, WEEK, 2 * WEEK)).await.unwrap(); // [0,w) and [w,2w) touch
    // changing a range may overlap its own old self, not the neighbour
    let mut wider = first.clone();
    wider.to = t(WEEK - 1);
    cs.set(wider).await.unwrap();
    let mut too_wide = first.clone();
    too_wide.to = t(WEEK + 1);
    assert!(cs.set(too_wide).await.is_err());
    assert_eq!(cs.mine(t(0), t(3 * WEEK)).await.unwrap().len(), 2);
    // removing makes room
    cs.remove(first.id).await.unwrap();
    cs.set(cap(100, 0, WEEK)).await.unwrap();
}

#[tokio::test]
async fn the_person_is_always_the_caller() {
    let (repo, _) = setup().await;
    let mut c = cap(600, 0, WEEK);
    c.person = person("somebody-else");
    let saved = repo.capacity().set(c).await.unwrap();
    assert_eq!(saved.person, common::ME);
}

#[tokio::test]
async fn whole_weeks_give_the_minutes_per_week() {
    let (repo, _) = setup().await;
    repo.capacity().set(cap(2400, 0, 4 * WEEK)).await.unwrap();
    let two = repo.capacity().load(t(0), t(2 * WEEK)).await.unwrap();
    assert_eq!(
        two,
        Load {
            capacity_minutes: 4800,
            allocated_minutes: 0
        }
    );
}

#[tokio::test]
async fn a_range_that_cuts_a_week_takes_its_share() {
    let (repo, _) = setup().await;
    repo.capacity().set(cap(2400, 0, 4 * WEEK)).await.unwrap();
    // 84 hours = half a week
    let half = repo.capacity().load(t(24), t(24 + 84)).await.unwrap();
    assert_eq!(half.capacity_minutes, 1200);
    // 1 day of 7
    let day = repo.capacity().load(t(30), t(54)).await.unwrap();
    assert_eq!(day.capacity_minutes, 342); // 2400 / 7 = 342.857..., rounded down
}

#[tokio::test]
async fn only_the_overlap_with_the_capacity_range_counts() {
    let (repo, _) = setup().await;
    repo.capacity().set(cap(2400, 0, WEEK)).await.unwrap();
    // half of the query lies before the capacity starts
    let got = repo.capacity().load(t(-84), t(84)).await.unwrap();
    assert_eq!(got.capacity_minutes, 1200);
    // none of it overlaps
    assert_eq!(
        repo.capacity().load(t(WEEK), t(2 * WEEK)).await.unwrap(),
        Load::default()
    );
}

#[tokio::test]
async fn ranges_with_different_rates_add_up() {
    let (repo, _) = setup().await;
    repo.capacity().set(cap(2400, 0, WEEK)).await.unwrap();
    repo.capacity().set(cap(600, WEEK, 3 * WEEK)).await.unwrap();
    let got = repo.capacity().load(t(0), t(3 * WEEK)).await.unwrap();
    assert_eq!(got.capacity_minutes, 2400 + 2 * 600);
    // the exact sum is divided once: 3.5 days of 2400 and 3.5 days of 600
    let cut = repo
        .capacity()
        .load(t(WEEK - 84), t(WEEK + 84))
        .await
        .unwrap();
    assert_eq!(cut.capacity_minutes, 1200 + 300);
}

#[tokio::test]
async fn allocations_count_pro_rata_by_their_own_range_once_accepted() {
    let (_, store) = setup().await;
    let (a, b) = (
        repo_for(&store, &person("a")),
        repo_for(&store, &person("b")),
    );
    // 600 minutes over 10 days; the query covers the second half of it and 10 more days
    let big = b
        .capacity()
        .propose(&person("a"), "t", t(0), t(240), 600)
        .await
        .unwrap();
    // 100 minutes over 3 days; the query covers the first day: 33.33 rounds down to 33
    let small = b
        .capacity()
        .propose(&person("a"), "u", t(0), t(72), 100)
        .await
        .unwrap();
    // declined and proposed ones never count
    let no = b
        .capacity()
        .propose(&person("a"), "v", t(0), t(24), 999)
        .await
        .unwrap();
    b.capacity()
        .propose(&person("a"), "w", t(0), t(24), 999)
        .await
        .unwrap();
    for (id, yes) in [(big.id, true), (small.id, true), (no.id, false)] {
        a.capacity().answer(id, yes).await.unwrap();
    }
    assert_eq!(
        a.capacity()
            .load(t(120), t(480))
            .await
            .unwrap()
            .allocated_minutes,
        300
    );
    assert_eq!(
        a.capacity()
            .load(t(0), t(24))
            .await
            .unwrap()
            .allocated_minutes,
        60 + 33
    );
    assert_eq!(
        a.capacity()
            .load(t(240), t(300))
            .await
            .unwrap()
            .allocated_minutes,
        0
    );
}
