//! Resources and reservations through the frontend's `Backend` trait, over the real core.
//!
//! Pattern: none (tests).

mod common;

use common::{T0, T1, T2, World, first, made, v};
use fs_admin_frontend::{Backend, BackendError, Values};
use fs_resources_core::describe::{RESERVATION, RESOURCE};
use fs_types::BasicType;
use fs_types::describe::Edit;

#[test]
fn a_resource_is_created_listed_changed_by_an_edit_and_deleted() {
    let w = World::new();
    let b = w.of(RESOURCE);
    let id = made(
        &b,
        "create",
        &v(&[
            ("kind", &["thing"]),
            ("name", &["Projector"]),
            ("quantity", &["2"]),
            ("unit", &["piece"]),
            ("owner", &["https://example.org/people/me"]),
        ]),
    );
    assert!(id.starts_with("https://example.org/resources/"));
    let all = b.list(&Values::new()).unwrap();
    assert_eq!(all.len(), 1);
    assert_eq!(first(&all[0], "name"), "Projector");
    assert_eq!(all[0].id, id);

    // missing = unchanged, no values = cleared, values = replaced; the quantity goes through its own call
    let edit = Edit::new()
        .set(&["labels"], BasicType::Keywords, &["av, loan"])
        .clear(&["owner"], BasicType::Reference)
        .set(&["quantity"], BasicType::Text, &["3"]);
    b.change(&id, &edit).unwrap();
    let now = &b.list(&Values::new()).unwrap()[0];
    assert_eq!(
        first(now, "name"),
        "Projector",
        "not in the edit: unchanged"
    );
    assert_eq!(first(now, "unit"), "piece");
    assert_eq!(first(now, "labels"), "av, loan");
    assert_eq!(first(now, "quantity"), "3");
    assert!(!now.values.contains_key("owner"), "empty = cleared");

    assert_eq!(b.list(&v(&[("term", &["proj"])])).unwrap().len(), 1);
    assert_eq!(b.list(&v(&[("term", &["zzz"])])).unwrap().len(), 0);
    assert_eq!(b.list(&v(&[("kind", &["room"])])).unwrap().len(), 0);

    b.run("delete", Some(&id), &Values::new()).unwrap();
    assert!(b.list(&Values::new()).unwrap().is_empty());
}

#[test]
fn reservations_cannot_overbook_and_finish_frees() {
    let w = World::new();
    let (res, rsv) = (w.of(RESOURCE), w.of(RESERVATION));
    let thing = made(
        &res,
        "create",
        &v(&[
            ("kind", &["thing"]),
            ("name", &["Tent"]),
            ("quantity", &["1"]),
        ]),
    );
    let ask = |resource: &str, holder: &str| {
        rsv.run(
            "request",
            None,
            &v(&[
                ("resource", &[resource]),
                ("quantity", &["1"]),
                ("from", &[T0]),
                ("to", &[T1]),
                ("holder", &[holder]),
            ]),
        )
    };
    let a = ask(&thing, "https://example.org/people/a")
        .unwrap()
        .id
        .unwrap();
    let b = ask(&thing, "https://example.org/people/b")
        .unwrap()
        .id
        .unwrap();
    rsv.run("confirm", Some(&a), &Values::new()).unwrap();
    // the core refuses the second confirmation: one tent only
    assert_eq!(
        rsv.run("confirm", Some(&b), &Values::new()),
        Err(BackendError::Rejected(Vec::new()))
    );

    let all = rsv.list(&Values::new()).unwrap();
    assert_eq!(all.len(), 2, "a list without a resource walks them all");
    assert_eq!(
        rsv.list(&v(&[("holder", &["https://example.org/people/a"])]))
            .unwrap()
            .len(),
        1
    );
    assert_eq!(rsv.list(&v(&[("resource", &[&thing])])).unwrap().len(), 2);

    let free = res
        .run("free", Some(&thing), &v(&[("from", &[T0]), ("to", &[T2])]))
        .unwrap();
    assert_eq!(
        free.values["free"],
        ["0", "1"],
        "none free while it is out, one after"
    );

    rsv.run("finish", Some(&a), &v(&[("quantity", &["1"])]))
        .unwrap();
    let a_now = rsv
        .list(&Values::new())
        .unwrap()
        .into_iter()
        .find(|r| r.id == a)
        .unwrap();
    assert_eq!(first(&a_now, "state"), "returned");
    rsv.run("confirm", Some(&b), &Values::new()).unwrap();

    // an edit of a field the core cannot change is refused with a finding on it, not dropped
    match rsv.change(&b, &Edit::new().set(&["note"], BasicType::Text, &["x"])) {
        Err(BackendError::Rejected(issues)) => assert_eq!(issues[0].field, "note"),
        other => panic!("{other:?}"),
    }
    rsv.change(&b, &Edit::new().set(&["to"], BasicType::DateTime, &[T2]))
        .unwrap();
}
