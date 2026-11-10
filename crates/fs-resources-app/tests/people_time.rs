//! Capacity and allocations through the frontend's `Backend` trait, over the real core.
//!
//! Pattern: none (tests).

mod common;

use common::{T0, T1, T2, World, first, made, v};
use fs_admin_frontend::{Backend, BackendError, Values};
use fs_resources_core::describe::{ALLOCATION, CAPACITY};
use fs_types::BasicType;
use fs_types::describe::Edit;

#[test]
fn capacity_and_allocations_act_for_the_person() {
    let w = World::new();
    let (cap, alloc) = (w.of(CAPACITY), w.of(ALLOCATION));
    let c = made(
        &cap,
        "set",
        &v(&[("from", &[T0]), ("to", &[T2]), ("minutesPerWeek", &["600"])]),
    );
    assert_eq!(
        cap.run(
            "set",
            None,
            &v(&[("from", &[T1]), ("to", &[T2]), ("minutesPerWeek", &["60"])])
        ),
        Err(BackendError::Rejected(Vec::new())),
        "overlap is refused"
    );
    cap.change(&c, &Edit::new().set(&["note"], BasicType::Text, &["mine"]))
        .unwrap();
    let rec = &cap.list(&Values::new()).unwrap()[0];
    assert_eq!(
        (
            first(rec, "note").as_str(),
            first(rec, "minutesPerWeek").as_str()
        ),
        ("mine", "600")
    );
    let load = cap
        .run("load", None, &v(&[("from", &[T0]), ("to", &[T2])]))
        .unwrap();
    assert!(load.values.contains_key("capacityMinutes"));

    let me = "https://example.org/people/me";
    alloc
        .run(
            "propose",
            None,
            &v(&[
                ("person", &[me]),
                ("subject", &["https://example.org/tasks/t"]),
                ("from", &[T0]),
                ("to", &[T1]),
                ("minutes", &["60"]),
            ]),
        )
        .unwrap();
    // asked of me and asked by me is the same record: listed once
    let list = alloc.list(&Values::new()).unwrap();
    assert_eq!(list.len(), 1);
    alloc
        .run("accept", Some(&list[0].id), &Values::new())
        .unwrap();
    assert_eq!(
        first(
            &alloc.list(&v(&[("scope", &["mine"])])).unwrap()[0],
            "state"
        ),
        "accepted"
    );
    alloc
        .run("withdraw", Some(&list[0].id), &Values::new())
        .unwrap();

    // without a groups program the group load says so; sharing and listing work
    let group = "urn:uuid:01900000-0000-7000-8000-000000000001";
    cap.run(
        "share",
        None,
        &v(&[("group", &[group]), ("share", &["on"])]),
    )
    .unwrap();
    assert_eq!(
        cap.run("sharedGroups", None, &Values::new())
            .unwrap()
            .values["groups"],
        [group]
    );
    assert_eq!(
        cap.run(
            "groupLoad",
            None,
            &v(&[("group", &[group]), ("from", &[T0]), ("to", &[T2])])
        ),
        Err(BackendError::Rejected(Vec::new()))
    );
    cap.run("delete", Some(&c), &Values::new()).unwrap();
}
