//! What a group sees of its members' time: rounded totals, from five who agreed, or nothing.

mod common;

use std::sync::Arc;

use common::{Roster, cap, person, repo_for, setup, t};
use fs_objects::ObjectId;
use fs_resources_core::{Error, GroupLoad, Load, MIN_SHARING, ResourceRepo};

const WEEK: i64 = 168;
/// Capacity of the five people, a week each: 1595 in all, which rounds to 1500 (steps of 300).
const MINUTES: [u32; 6] = [130, 250, 310, 400, 505, 90];

struct Club {
    store: common::Stores,
    asker: ResourceRepo,
    x: ObjectId,
    y: ObjectId,
}

/// Six people p0..p5 with capacity, all members of X and Y; nobody shares yet.
async fn club() -> Club {
    let (_, store) = setup().await;
    let (x, y) = (ObjectId::new(), ObjectId::new());
    let people: Vec<String> = (0..6).map(|i| person(&format!("p{i}"))).collect();
    // the asker is a member too, with no capacity and not sharing
    let mut everyone = people.clone();
    everyone.push(person("asker"));
    let mut roster = Roster::default();
    roster.0.insert(x, everyone.clone());
    roster.0.insert(y, everyone);
    for (i, p) in people.iter().enumerate() {
        repo_for(&store, p)
            .capacity()
            .set(cap(MINUTES[i], 0, WEEK))
            .await
            .unwrap();
    }
    let asker = repo_for(&store, &person("asker")).with_groups(Arc::new(roster));
    Club { store, asker, x, y }
}

impl Club {
    async fn share(&self, who: usize, group: ObjectId, on: bool) {
        let me = repo_for(&self.store, &person(&format!("p{who}")));
        me.capacity().share_with_group(group, on).await.unwrap();
    }
    async fn seen(&self, group: ObjectId) -> GroupLoad {
        self.asker
            .capacity()
            .group_load(group, t(0), t(WEEK))
            .await
            .unwrap()
    }
}

#[tokio::test]
async fn without_a_group_lookup_it_is_unsupported() {
    let (repo, _) = setup().await;
    let err = repo
        .capacity()
        .group_load(ObjectId::new(), t(0), t(WEEK))
        .await
        .unwrap_err();
    assert!(matches!(err, Error::Unsupported(_)));
    assert!(!repo.items().capabilities().is_supported("capacity.group"));
}

#[tokio::test]
async fn four_sharing_people_are_hidden() {
    let c = club().await;
    for i in 0..MIN_SHARING - 1 {
        c.share(i, c.x, true).await;
    }
    let got = c.seen(c.x).await;
    assert_eq!(
        got,
        GroupLoad {
            hidden: true,
            people: 0,
            allocations_hidden: false,
            load: Load::default()
        }
    );
    assert!(
        c.asker
            .items()
            .capabilities()
            .is_supported("capacity.group")
    );
}

#[tokio::test]
async fn five_sharing_people_show_rounded_totals() {
    let c = club().await;
    for i in 0..5 {
        c.share(i, c.x, true).await;
    }
    let got = c.seen(c.x).await;
    assert!(!got.hidden);
    assert_eq!(got.people, 0, "a count would show who is absent");
    assert_eq!(got.load.capacity_minutes, 1500); // 1595 to the nearest 60 * 5 = 300
    assert_eq!(got.load.allocated_minutes, 0);
    // one person with an allocation is too few: it is hidden, not rounded
    let ask = repo_for(&c.store, &person("boss"))
        .capacity()
        .propose(&person("p0"), "task", t(0), t(WEEK), 200)
        .await
        .unwrap();
    repo_for(&c.store, &person("p0"))
        .capacity()
        .answer(ask.id, true)
        .await
        .unwrap();
    let seen = c.seen(c.x).await;
    assert!(seen.allocations_hidden);
    assert_eq!(seen.load.allocated_minutes, 0);
    assert_eq!(seen.load.capacity_minutes, 1500);
}

#[tokio::test]
async fn a_person_who_shares_but_is_not_a_member_does_not_count() {
    let c = club().await;
    for i in 0..4 {
        c.share(i, c.x, true).await;
    }
    let outsider = repo_for(&c.store, &person("outsider"));
    outsider.capacity().set(cap(600, 0, WEEK)).await.unwrap();
    outsider
        .capacity()
        .share_with_group(c.x, true)
        .await
        .unwrap();
    assert!(c.seen(c.x).await.hidden);
}

#[tokio::test]
async fn a_member_who_does_not_share_does_not_count() {
    let c = club().await;
    for i in 0..4 {
        c.share(i, c.x, true).await;
    }
    // p4 is a member with capacity, but never switched sharing on
    assert!(c.seen(c.x).await.hidden);
    c.share(4, c.x, true).await;
    assert!(!c.seen(c.x).await.hidden);
    c.share(4, c.x, false).await;
    assert!(c.seen(c.x).await.hidden);
}

#[tokio::test]
async fn sharing_with_one_group_does_not_count_for_another() {
    let c = club().await;
    for i in 0..5 {
        c.share(i, c.x, true).await;
    }
    assert!(!c.seen(c.x).await.hidden);
    assert!(
        c.seen(c.y).await.hidden,
        "all five are members of Y but shared with X only"
    );
    let me = repo_for(&c.store, &person("p0"));
    assert_eq!(me.capacity().shared_groups().await.unwrap(), vec![c.x]);
}

#[tokio::test]
async fn a_one_minute_change_in_one_capacity_does_not_change_the_total() {
    let c = club().await;
    for i in 0..5 {
        c.share(i, c.x, true).await;
    }
    let before = c.seen(c.x).await;
    let p4 = repo_for(&c.store, &person("p4"));
    let mut mine = p4.capacity().mine(t(0), t(WEEK)).await.unwrap().remove(0);
    mine.minutes_per_week += 1;
    p4.capacity().set(mine).await.unwrap();
    assert_eq!(
        p4.capacity()
            .load(t(0), t(WEEK))
            .await
            .unwrap()
            .capacity_minutes,
        506
    );
    assert_eq!(c.seen(c.x).await, before);
}

#[tokio::test]
async fn sharing_is_off_until_the_person_says_so() {
    let c = club().await;
    for i in 0..6 {
        let me = repo_for(&c.store, &person(&format!("p{i}")));
        assert!(me.capacity().shared_groups().await.unwrap().is_empty());
    }
    assert!(c.seen(c.x).await.hidden);
}
