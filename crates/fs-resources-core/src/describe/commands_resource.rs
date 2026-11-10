//! Commands of Resource: rows of the one table (`commands.rs`).
//!
//! Pattern: Registry as data — a part of the table, split by data type to keep files short.

use fs_types::BasicType;
use fs_types::CommandKind as K;

use super::commands::{CLASSES, CURSOR, NEXT, R_CHECK, R_CLASS, R_LINK, R_STOCK, found, id};
use super::fields::{ANY, KINDS, PERSON, PLACE, RESERVATION, RESOURCE};
use super::spec::{CommandSpec, FieldSpec, choice, reference, text, when};

pub(super) const RESOURCE_COMMANDS: &[CommandSpec] = &[
    CommandSpec::done(
        RESOURCE,
        "create",
        K::Create,
        "resource.create",
        &[
            choice("kind", KINDS).req(),
            text("name").req(),
            text("quantity").req(),
            text("unit"),
            reference("owner", PERSON),
            reference("place", PLACE),
        ],
        &[reference("id", RESOURCE).req()],
    ),
    CommandSpec::done(RESOURCE, "change", K::Change, "resource.edit", &[], &[]),
    CommandSpec::done(
        RESOURCE,
        "find",
        K::Find,
        "resource.read",
        &[
            text("term"),
            choice("kind", KINDS),
            reference("owner", PERSON),
            reference("place", PLACE),
            text("label"),
            CURSOR,
        ],
        &[found("items", RESOURCE), NEXT],
    ),
    CommandSpec::done(
        RESOURCE,
        "delete",
        K::Delete,
        "resource.delete",
        &[id(RESOURCE)],
        &[],
    ),
    CommandSpec::done(
        RESOURCE,
        "free",
        K::Other,
        "reservation.read",
        &[id(RESOURCE), when("from").req(), when("to").req()],
        &[when("from").many(), when("to").many(), text("free").many()],
    ),
    CommandSpec::missing(
        RESOURCE,
        "setMinimum",
        K::Other,
        R_STOCK,
        &[
            id(RESOURCE),
            text("minimum").req(),
            FieldSpec::new("dueDate", BasicType::Date),
        ],
        &[],
    ),
    CommandSpec::missing(
        RESOURCE,
        "missing",
        K::Find,
        R_STOCK,
        &[CURSOR],
        &[found("items", RESOURCE), NEXT],
    ),
    CommandSpec::missing(
        RESOURCE,
        "checkOut",
        K::Other,
        R_CHECK,
        &[id(RESOURCE), reference("holder", PERSON).req(), when("to")],
        &[reference("reservation", RESERVATION)],
    ),
    CommandSpec::missing(RESOURCE, "checkIn", K::Other, R_CHECK, &[id(RESOURCE)], &[]),
    CommandSpec::missing(
        RESOURCE,
        "setStatus",
        K::Other,
        R_CHECK,
        &[id(RESOURCE), text("status").req()],
        &[],
    ),
    CommandSpec::missing(
        RESOURCE,
        "setClass",
        K::Other,
        R_CLASS,
        &[id(RESOURCE), choice("class", CLASSES).req()],
        &[],
    ),
    CommandSpec::missing(
        RESOURCE,
        "link",
        K::Other,
        R_LINK,
        &[id(RESOURCE), reference("target", ANY).req()],
        &[],
    ),
];
