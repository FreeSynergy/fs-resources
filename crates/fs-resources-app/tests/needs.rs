//! Needs and offers through the frontend's `Backend` trait, over the real core.
//!
//! Pattern: none (tests).

mod common;

use common::{T0, T1, World, first, made, v};
use fs_admin_frontend::{Backend, Values};
use fs_resources_core::describe::{NEED, OFFER, RESERVATION, RESOURCE};
use fs_types::BasicType;
use fs_types::describe::Edit;

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
