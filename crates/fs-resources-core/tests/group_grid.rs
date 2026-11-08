//! A group total sits on a fixed grid, thresholds allocations too, and counts a person once.

mod common;

use std::sync::Arc;

use common::{Roster, Stores, cap, person, repo_for, setup, t};
use fs_objects::traits::UnloggedObjects;
use fs_objects::{Envelope, ObjectId, StoredObject, Timestamp};
use fs_resources_core::model::{Record, document};
use fs_resources_core::{Error, GroupShare, ResourceRepo};

const WEEK: i64 = 168;

/// `n` people p0.. with `mpw` minutes a week from the start of time; the first `sharing` share.
async fn group(s: &Stores, n: usize, sharing: usize, mpw: u32) -> (ObjectId, ResourceRepo) {
    let g = ObjectId::new();
    let mut people: Vec<String> = (0..n).map(|i| person(&format!("p{i}"))).collect();
    for (i, p) in people.iter().enumerate() {
        let r = repo_for(s, p);
        r.capacity()
            .set(cap(mpw + i as u32, -1000 * WEEK, 1000 * WEEK))
            .await
            .unwrap();
        if i < sharing {
            r.capacity().share_with_group(g, true).await.unwrap();
        }
    }
    people.push(person("member"));
    let mut roster = Roster::default();
    roster.0.insert(g, people);
    (
        g,
        repo_for(s, &person("member")).with_groups(Arc::new(roster)),
    )
}

#[tokio::test]
async fn from_must_be_a_monday_at_midnight_utc() {
    let (_, s) = setup().await;
    let (g, member) = group(&s, 5, 5, 300).await;
    assert!(member.capacity().group_load(g, t(0), t(WEEK)).await.is_ok());
    let off = |secs: i64| t(0) + chrono::Duration::seconds(secs);
    for from in [
        off(1),
        off(-1),
        off(3600),
        t(24),
        t(-24),
        off(86_400 * 7 - 1),
    ] {
        let r = member
            .capacity()
            .group_load(g, from, from + chrono::Duration::weeks(1))
            .await;
        assert!(matches!(r, Err(Error::Invalid(_))), "{from}: {r:?}");
    }
}

#[tokio::test]
async fn a_search_on_from_learns_nothing_finer_than_the_step() {
    let (_, s) = setup().await;
    let (g, member) = group(&s, 5, 5, 300).await; // 300..304 a week, 1510 in all
    let mut accepted = vec![];
    for secs in (0..86_400 * 7).step_by(3607) {
        let from = t(0) + chrono::Duration::seconds(secs);
        let to = from + chrono::Duration::weeks(1);
        if let Ok(got) = member.capacity().group_load(g, from, to).await {
            accepted.push((secs, got.load.capacity_minutes));
        }
    }
    assert_eq!(
        accepted,
        vec![(0, 1500)],
        "only the grid point answers, with a step"
    );
}

#[tokio::test]
async fn the_ends_of_time_are_refused_without_a_panic() {
    let (_, s) = setup().await;
    let (g, member) = group(&s, 5, 5, 300).await;
    let max = Timestamp::MAX_UTC;
    let min = Timestamp::MIN_UTC;
    for (from, to) in [
        (max, max),
        (min, max),
        (t(0), max),
        (max - chrono::Duration::days(3), max),
        (min, t(0)),
    ] {
        let r = member.capacity().group_load(g, from, to).await;
        assert!(matches!(r, Err(Error::Invalid(_))), "{from} {to}: {r:?}");
    }
}

async fn allocate(s: &Stores, people: usize) {
    for i in 0..people {
        let p = person(&format!("p{i}"));
        let ask = repo_for(s, &person("boss"))
            .capacity()
            .propose(&p, "t", t(0), t(WEEK), 1200)
            .await
            .unwrap();
        repo_for(s, &p)
            .capacity()
            .answer(ask.id, true)
            .await
            .unwrap();
    }
}

#[tokio::test]
async fn allocations_need_five_people_too() {
    let (_, s) = setup().await;
    let (g, member) = group(&s, 6, 6, 300).await;
    allocate(&s, 1).await; // one member's 1200 must not show as 1200
    let got = member
        .capacity()
        .group_load(g, t(0), t(WEEK))
        .await
        .unwrap();
    assert!(!got.hidden && got.allocations_hidden);
    assert_eq!(got.load.allocated_minutes, 0);
    assert!(got.load.capacity_minutes > 0);
    allocate(&s, 4).await; // p0 again plus three more: 4 people in all
    let got = member
        .capacity()
        .group_load(g, t(0), t(WEEK))
        .await
        .unwrap();
    assert!(got.allocations_hidden && got.load.allocated_minutes == 0);
    allocate(&s, 5).await;
    let got = member
        .capacity()
        .group_load(g, t(0), t(WEEK))
        .await
        .unwrap();
    assert!(!got.allocations_hidden);
    assert_eq!(got.load.allocated_minutes % 300, 0);
    assert!(got.load.allocated_minutes > 0);
}

#[tokio::test]
async fn a_person_with_several_share_records_counts_once() {
    let (_, s) = setup().await;
    let (g, member) = group(&s, 5, 4, 300).await; // four share; p4 does not
    let plant = |extra: usize| {
        let s = &s;
        async move {
            for _ in 0..extra {
                let share = GroupShare {
                    id: ObjectId::new(),
                    person: person("p0"),
                    group: g,
                };
                let mut e = Envelope::new(GroupShare::TYPE, "share");
                e.id = share.id;
                e.context = Some("resources".into());
                let doc = document(&share, "https://example.org/resources/x").unwrap();
                let o = StoredObject::new(e, "application/activity+json", doc)
                    .with_index("person", person("p0"))
                    .with_index("group", g.to_string());
                s.private.put_unlogged(o).await.unwrap();
            }
        }
    };
    plant(3).await; // p0 now has four records, in all seven for four people
    let got = member
        .capacity()
        .group_load(g, t(0), t(WEEK))
        .await
        .unwrap();
    assert!(got.hidden, "four people, however many records");
}
