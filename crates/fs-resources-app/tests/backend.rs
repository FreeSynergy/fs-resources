//! The backend against the real core (in-memory stores), only through the frontend's `Backend` trait.
//!
//! Pattern: none (tests). No test double: the same objects the program uses.

use std::sync::Arc;

use fs_admin_frontend::{Backend, BackendError, Record, Values};
use fs_objects::TursoStore;
use fs_resources_app::{CoreBackend, Ports};
use fs_resources_core::ResourceRepo;
use fs_resources_core::describe::{
    ALLOCATION, CAPACITY, COMMANDS, NEED, OFFER, RESERVATION, RESOURCE, unsupported_commands,
};
use fs_types::describe::Edit;
use fs_types::{BasicType, CommandKind};

struct World {
    rt: tokio::runtime::Runtime,
    ports: Ports,
}

impl World {
    fn new() -> Self {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let repo = rt.block_on(async {
            let main = Arc::new(TursoStore::open_in_memory().await.unwrap());
            let private = Arc::new(TursoStore::open_in_memory().await.unwrap());
            ResourceRepo::new(main, private, "https://example.org/people/me")
                .with_instance("example.org")
        });
        Self {
            ports: Ports::from_repo(&repo),
            rt,
        }
    }

    fn of(&self, data_type: &'static str) -> CoreBackend {
        CoreBackend::new(data_type, self.ports.clone(), self.rt.handle().clone())
    }
}

fn v(pairs: &[(&str, &[&str])]) -> Values {
    pairs
        .iter()
        .map(|(k, vs)| {
            (
                (*k).to_owned(),
                vs.iter().map(|s| (*s).to_owned()).collect(),
            )
        })
        .collect()
}

fn first(r: &Record, field: &str) -> String {
    r.values
        .get(field)
        .and_then(|x| x.first())
        .cloned()
        .unwrap_or_default()
}

fn made(b: &CoreBackend, name: &str, input: &Values) -> String {
    b.run(name, None, input)
        .unwrap()
        .id
        .expect("the command names what it made")
}

const T0: &str = "2026-03-02T09:00:00Z";
const T1: &str = "2026-03-02T17:00:00Z";
const T2: &str = "2026-03-03T09:00:00Z";

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

#[test]
fn an_accepted_offer_makes_the_reservation() {
    let w = World::new();
    let (res, rsv, need, offer) = (w.of(RESOURCE), w.of(RESERVATION), w.of(NEED), w.of(OFFER));
    let tent = made(
        &res,
        "create",
        &v(&[
            ("kind", &["thing"]),
            ("name", &["Tent"]),
            ("quantity", &["3"]),
        ]),
    );
    let n = made(
        &need,
        "create",
        &v(&[
            ("what", &["tents"]),
            ("quantity", &["2"]),
            ("from", &[T0]),
            ("to", &[T1]),
            ("subject", &["https://example.org/tasks/camp"]),
        ]),
    );
    assert_eq!(need.list(&v(&[("scope", &["open"])])).unwrap().len(), 1);
    let o = made(
        &offer,
        "create",
        &v(&[
            ("need", &[&n]),
            ("by", &["https://example.org/people/a"]),
            ("quantity", &["2"]),
            ("resource", &[&tent]),
        ]),
    );
    assert_eq!(
        offer.list(&Values::new()).unwrap().len(),
        1,
        "a list without a need walks all needs"
    );
    offer.run("accept", Some(&o), &Values::new()).unwrap();
    let covered = &need.list(&Values::new()).unwrap()[0];
    assert_eq!(
        (
            first(covered, "state").as_str(),
            first(covered, "covered").as_str()
        ),
        ("covered", "2")
    );
    assert_eq!(rsv.list(&Values::new()).unwrap().len(), 1);
    offer.run("withdraw", Some(&o), &Values::new()).unwrap();
    assert_eq!(
        first(&need.list(&Values::new()).unwrap()[0], "state"),
        "open"
    );
    need.change(
        &n,
        &Edit::new()
            .clear(&["to"], BasicType::DateTime)
            .clear(&["from"], BasicType::DateTime),
    )
    .unwrap();
    assert!(
        !need.list(&Values::new()).unwrap()[0]
            .values
            .contains_key("from")
    );
    need.run("close", Some(&n), &Values::new()).unwrap();
    need.run("delete", Some(&n), &Values::new()).unwrap();
}

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

#[test]
fn commands_the_core_does_not_have_are_refused_by_name_and_reported_unsupported() {
    let w = World::new();
    let missing: Vec<_> = unsupported_commands().collect();
    assert!(missing.len() >= 7);
    for c in missing {
        let b = w.of(c.of);
        assert!(!b.supported(c.name), "{}.{}", c.of, c.name);
        let refusal = b
            .run(
                c.name,
                Some("https://example.org/resources/x"),
                &Values::new(),
            )
            .unwrap_err();
        match refusal {
            BackendError::Rejected(issues) => {
                assert_eq!(issues.len(), 1);
                assert_eq!(issues[0].field, c.name, "the finding names the command");
            }
            other => panic!("{}.{}: {other:?}", c.of, c.name),
        }
    }
}

#[test]
fn every_supported_command_of_the_table_has_a_handler() {
    // an empty call may be refused for its input, never answered "no such command"
    let w = World::new();
    for c in COMMANDS.iter().filter(|c| c.supported) {
        let b = w.of(c.of);
        assert!(b.supported(c.name));
        if c.kind == CommandKind::Find {
            b.list(&Values::new())
                .unwrap_or_else(|e| panic!("{}.{}: {e:?}", c.of, c.name));
        } else {
            assert_ne!(
                b.run(c.name, None, &Values::new()),
                Err(BackendError::NotFound),
                "{}.{}",
                c.of,
                c.name
            );
        }
    }
    assert_eq!(
        w.of(RESOURCE).run("nonsense", None, &Values::new()),
        Err(BackendError::NotFound)
    );
}
