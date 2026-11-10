//! Commands of Reservation: rows of the one table (`commands.rs`).
//!
//! Pattern: Registry as data — a part of the table, split by data type to keep files short.

use fs_types::CommandKind as K;

use super::commands::{CURSOR, NEXT, found, id};
use super::fields::{PERSON, RESERVATION, RESOURCE, TASK};
use super::spec::{CommandSpec, reference, text, when};

pub(super) const RESERVATION_COMMANDS: &[CommandSpec] = &[
    CommandSpec::done(
        RESERVATION,
        "request",
        K::Create,
        "reservation.request",
        &[
            reference("resource", RESOURCE).req(),
            text("quantity").req(),
            when("from").req(),
            when("to").req(),
            reference("holder", PERSON).req(),
            reference("subject", TASK),
            text("note"),
        ],
        &[reference("id", RESERVATION).req()],
    ),
    CommandSpec::done(
        RESERVATION,
        "change",
        K::Change,
        "reservation.request",
        &[],
        &[],
    ),
    CommandSpec::done(
        RESERVATION,
        "find",
        K::Find,
        "reservation.read",
        &[
            reference("resource", RESOURCE),
            reference("holder", PERSON),
            when("from"),
            when("to"),
            CURSOR,
        ],
        &[found("items", RESERVATION), NEXT],
    ),
    CommandSpec::done(
        RESERVATION,
        "confirm",
        K::Other,
        "reservation.decide",
        &[id(RESERVATION)],
        &[],
    ),
    CommandSpec::done(
        RESERVATION,
        "decline",
        K::Other,
        "reservation.decide",
        &[id(RESERVATION)],
        &[],
    ),
    CommandSpec::done(
        RESERVATION,
        "cancel",
        K::Other,
        "reservation.request",
        &[id(RESERVATION)],
        &[],
    ),
    CommandSpec::done(
        RESERVATION,
        "finish",
        K::Other,
        "reservation.decide",
        &[id(RESERVATION), text("quantity").req()],
        &[],
    ),
];
