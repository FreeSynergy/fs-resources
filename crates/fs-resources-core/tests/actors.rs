//! Two people on one program share its locks, through `as_actor`.
#![allow(clippy::assert_is_empty, clippy::many_single_char_names)] // tests: plain `is_empty()` asserts read clearer than typed empty comparisons; tests: short bindings (a, b, x, y) are the clear names here

mod common;

use common::{ask, make, person, setup, stores};
use fs_objects::ObjectId;
use fs_resources_core::ResourceKind;

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn two_actors_cannot_both_confirm_the_same_single_tent() {
    let (a, _) = setup().await;
    let b = a.as_actor(person("b"));
    for _ in 0..30 {
        let tent = make(&a, ResourceKind::Thing, 1).await;
        let r1 = ask(&a, &tent, 1, 0, 24).await;
        let r2 = ask(&b, &tent, 1, 0, 24).await;
        let (x, y) = tokio::join!(
            a.reservations().confirm(r1.id),
            b.reservations().confirm(r2.id)
        );
        assert!(!(x.is_ok() && y.is_ok()), "both confirmed the only tent");
        assert!(x.is_ok() || y.is_ok());
    }
}

#[tokio::test]
async fn as_actor_acts_for_the_other_person_on_the_same_data() {
    let (a, _) = setup().await;
    let b = a.as_actor(person("b"));
    let c = common::cap(600, 0, 168);
    a.capacity().set(c).await.unwrap();
    assert_eq!(
        a.capacity()
            .mine(common::t(0), common::t(168))
            .await
            .unwrap()
            .len(),
        1
    );
    assert!(
        b.capacity()
            .mine(common::t(0), common::t(168))
            .await
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        a.capacity()
            .load(common::t(0), common::t(168))
            .await
            .unwrap()
            .capacity_minutes,
        600
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn one_person_sharing_twice_at_once_keeps_one_share() {
    let (a, _) = setup().await;
    let g = ObjectId::new();
    for _ in 0..30 {
        let (x, y) = (a.as_actor(person("p")), a.as_actor(person("p")));
        let (u, v) = tokio::join!(
            x.capacity().share_with_group(g, true),
            y.capacity().share_with_group(g, true)
        );
        u.unwrap();
        v.unwrap();
        x.capacity().share_with_group(g, false).await.unwrap();
        assert!(x.capacity().shared_groups().await.unwrap().is_empty());
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn stopping_removes_duplicates_that_separate_repos_made() {
    // separate repos share no lock, so duplicates can appear; stopping must still end sharing
    let s = stores().await;
    let g = ObjectId::new();
    let p = person("p");
    for _ in 0..30 {
        let (x, y) = (common::repo_for(&s, &p), common::repo_for(&s, &p));
        let (u, v) = tokio::join!(
            x.capacity().share_with_group(g, true),
            y.capacity().share_with_group(g, true)
        );
        u.unwrap();
        v.unwrap();
        x.capacity().share_with_group(g, false).await.unwrap();
        assert!(!x.capacity().shared_groups().await.unwrap().contains(&g));
    }
}
