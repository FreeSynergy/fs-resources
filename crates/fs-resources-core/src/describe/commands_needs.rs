//! Commands of Need and Offer: rows of the one table (`commands.rs`).
//!
//! Pattern: Registry as data — a part of the table, split by data type to keep files short.

use fs_types::CommandKind as K;

use super::commands::{CURSOR, NEXT, SCOPE_NEEDS, found, id};
use super::fields::{NEED, OFFER, PERSON, RESOURCE, TASK};
use super::spec::{CommandSpec, choice, reference, text, when};

pub(super) const NEED_COMMANDS: &[CommandSpec] = &[
    CommandSpec::done(
        NEED,
        "create",
        K::Create,
        "need.create",
        &[
            text("what").req(),
            text("quantity").req(),
            text("unit"),
            when("from"),
            when("to"),
            reference("subject", TASK),
        ],
        &[reference("id", NEED).req()],
    ),
    CommandSpec::done(NEED, "change", K::Change, "need.create", &[], &[]),
    CommandSpec::done(
        NEED,
        "find",
        K::Find,
        "need.read",
        &[
            reference("subject", TASK),
            choice("scope", SCOPE_NEEDS),
            CURSOR,
        ],
        &[found("items", NEED), NEXT],
    ),
    CommandSpec::done(NEED, "close", K::Other, "need.create", &[id(NEED)], &[]),
    CommandSpec::done(NEED, "delete", K::Delete, "need.create", &[id(NEED)], &[]),
    // ---------------------------------------------------------------- Offer
    CommandSpec::done(
        OFFER,
        "create",
        K::Create,
        "offer.make",
        &[
            reference("need", NEED).req(),
            reference("by", PERSON).req(),
            text("quantity").req(),
            reference("resource", RESOURCE),
            text("note"),
        ],
        &[reference("id", OFFER).req()],
    ),
    CommandSpec::done(
        OFFER,
        "find",
        K::Find,
        "need.read",
        &[reference("need", NEED), CURSOR],
        &[found("items", OFFER), NEXT],
    ),
    CommandSpec::done(OFFER, "withdraw", K::Other, "offer.make", &[id(OFFER)], &[]),
    CommandSpec::done(OFFER, "accept", K::Other, "offer.decide", &[id(OFFER)], &[]),
    CommandSpec::done(
        OFFER,
        "decline",
        K::Other,
        "offer.decide",
        &[id(OFFER)],
        &[],
    ),
];
