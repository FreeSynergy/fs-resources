//! A group total reveals nothing finer than a step: members only, whole weeks, five contributors.

mod common;

use std::sync::Arc;

use common::{Roster, Stores, cap, person, repo_for, setup, t};
use fs_objects::ObjectId;
use fs_resources_core::{Error, GROUP_ROUND, GroupLoad, Load, ResourceRepo};

const WEEK: i64 = 168;

/// People p0.. with `(minutes per week, weeks of capacity)`; all share with the group and are
/// members. Returns the group, a member who asks, and a stranger who asks.
async fn group(s: &Stores, caps: &[(u32, i64)]) -> (ObjectId, ResourceRepo, ResourceRepo) {
    let g = ObjectId::new();
    let mut people: Vec<String> = (0..caps.len()).map(|i| person(&format!("p{i}"))).collect();
    for (i, p) in people.iter().enumerate() {
        let r = repo_for(s, p);
        if caps[i].0 > 0 {
            r.capacity()
                .set(cap(caps[i].0, 0, caps[i].1 * WEEK))
                .await
                .unwrap();
        }
        r.capacity().share_with_group(g, true).await.unwrap();
    }
    people.push(person("member"));
    let mut roster = Roster::default();
    roster.0.insert(g, people);
    let roster = Arc::new(roster);
    let member = repo_for(s, &person("member")).with_groups(roster.clone());
    (
        g,
        member,
        repo_for(s, &person("stranger")).with_groups(roster),
    )
}

fn hidden() -> GroupLoad {
    GroupLoad {
        hidden: true,
        people: 0,
        allocations_hidden: false,
        load: Load::default(),
    }
}

const FIVE: [(u32, i64); 5] = [
    (130, 1000),
    (251, 1000),
    (310, 1000),
    (400, 1000),
    (505, 1000),
];

#[tokio::test]
async fn a_stranger_gets_the_answer_for_too_few() {
    let (_, s) = setup().await;
    let (g, member, stranger) = group(&s, &FIVE).await;
    let seen = member
        .capacity()
        .group_load(g, t(0), t(WEEK))
        .await
        .unwrap();
    assert!(!seen.hidden);
    assert_eq!(
        stranger
            .capacity()
            .group_load(g, t(0), t(WEEK))
            .await
            .unwrap(),
        hidden()
    );
}

#[tokio::test]
async fn one_contributor_among_empty_sharers_stays_hidden_however_long_the_range() {
    let (_, s) = setup().await;
    let (g, member, _) = group(&s, &[(317, 1000), (0, 1), (0, 1), (0, 1), (0, 1)]).await;
    for weeks in [1, 2, 52, 520] {
        let got = member
            .capacity()
            .group_load(g, t(0), t(weeks * WEEK))
            .await
            .unwrap();
        assert_eq!(got, hidden(), "{weeks} weeks");
    }
}

#[tokio::test]
async fn only_whole_weeks_from_the_start_are_allowed() {
    let (_, s) = setup().await;
    let (g, member, _) = group(&s, &FIVE).await;
    let min = |m: i64| common::min(m);
    for to in [
        min(1),
        t(WEEK - 1),
        t(WEEK + 1),
        min(WEEK * 60 + 1),
        t(0),
        t(-WEEK),
    ] {
        let r = member.capacity().group_load(g, t(0), to).await;
        assert!(matches!(r, Err(Error::Invalid(_))), "{to}: {r:?}");
    }
    let r = member.capacity().group_load(g, t(0), t(521 * WEEK)).await;
    assert!(matches!(r, Err(Error::Invalid(_))));
    assert!(
        member
            .capacity()
            .group_load(g, t(0), t(520 * WEEK))
            .await
            .is_ok()
    );
}

#[tokio::test]
async fn a_search_on_the_end_of_the_range_learns_nothing_finer_than_the_step() {
    let (_, s) = setup().await;
    let (g, member, _) = group(&s, &FIVE).await; // 1596 minutes a week in all
    let mut seen = vec![];
    for weeks in 1..=60 {
        let got = member
            .capacity()
            .group_load(g, t(0), t(weeks * WEEK))
            .await
            .unwrap();
        assert_eq!(got.load.capacity_minutes % GROUP_ROUND, 0);
        seen.push(got.load.capacity_minutes);
    }
    // every week is rounded on its own, so n weeks give n times the same rounded week
    assert!(
        seen.iter().zip(1u64..).all(|(v, n)| *v == n * 1500),
        "{seen:?}"
    );
    // and any end that is not a whole week is refused, so the rate cannot be bisected
    for secs in [1, 59, 3600, 7 * 24 * 3600 - 1, 7 * 24 * 3600 + 1] {
        let to = t(0) + chrono::Duration::seconds(secs);
        assert!(member.capacity().group_load(g, t(0), to).await.is_err());
    }
}

#[tokio::test]
async fn one_week_with_fewer_than_five_contributors_hides_everything() {
    let (_, s) = setup().await;
    // the fifth person has capacity for two weeks only
    let mut caps = FIVE;
    caps[4].1 = 2;
    let (g, member, _) = group(&s, &caps).await;
    assert!(
        !member
            .capacity()
            .group_load(g, t(0), t(2 * WEEK))
            .await
            .unwrap()
            .hidden
    );
    assert_eq!(
        member
            .capacity()
            .group_load(g, t(0), t(3 * WEEK))
            .await
            .unwrap(),
        hidden()
    );
}

#[tokio::test]
async fn people_is_always_zero_so_a_week_without_someone_shows_no_count() {
    let (_, s) = setup().await;
    let mut caps = [(300, 1000); 7];
    caps[5].1 = 1;
    caps[6].1 = 1;
    let (g, member, _) = group(&s, &caps).await;
    for weeks in [1, 2] {
        let got = member
            .capacity()
            .group_load(g, t(0), t(weeks * WEEK))
            .await
            .unwrap();
        assert_eq!((got.hidden, got.people), (false, 0));
    }
}
