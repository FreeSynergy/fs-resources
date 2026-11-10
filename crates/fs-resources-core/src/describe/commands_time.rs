//! Commands of Capacity and Allocation (a person's own time): rows of the one table (`commands.rs`).
//!
//! Pattern: Registry as data — a part of the table, split by data type to keep files short.

use fs_types::CommandKind as K;

use super::commands::{CURSOR, NEXT, ON_OFF, WHO_ALLOC, found, id};
use super::fields::{ALLOCATION, CAPACITY, PERSON, TASK};
use super::spec::{CommandSpec, choice, reference, text, when};

pub(super) const TIME_COMMANDS: &[CommandSpec] = &[
    CommandSpec::done(
        CAPACITY,
        "set",
        K::Create,
        "capacity.set",
        &[
            when("from").req(),
            when("to").req(),
            text("minutesPerWeek").req(),
            text("note"),
        ],
        &[reference("id", CAPACITY).req()],
    ),
    CommandSpec::done(CAPACITY, "change", K::Change, "capacity.set", &[], &[]),
    CommandSpec::done(
        CAPACITY,
        "find",
        K::Find,
        "capacity.read",
        &[when("from"), when("to"), CURSOR],
        &[found("items", CAPACITY), NEXT],
    ),
    CommandSpec::done(
        CAPACITY,
        "delete",
        K::Delete,
        "capacity.set",
        &[id(CAPACITY)],
        &[],
    ),
    CommandSpec::done(
        CAPACITY,
        "load",
        K::Other,
        "capacity.read",
        &[when("from").req(), when("to").req()],
        &[text("capacityMinutes"), text("allocatedMinutes")],
    ),
    CommandSpec::done(
        CAPACITY,
        "share",
        K::Other,
        "capacity.share",
        &[
            reference("group", "Group").req(),
            choice("share", ON_OFF).req(),
        ],
        &[],
    ),
    CommandSpec::done(
        CAPACITY,
        "sharedGroups",
        K::Other,
        "capacity.share",
        &[],
        &[reference("groups", "Group").many()],
    ),
    CommandSpec::done(
        CAPACITY,
        "groupLoad",
        K::Other,
        "capacity.group",
        &[
            reference("group", "Group").req(),
            when("from").req(),
            when("to").req(),
        ],
        &[
            text("hidden"),
            text("allocationsHidden"),
            text("capacityMinutes"),
            text("allocatedMinutes"),
        ],
    ),
    // ---------------------------------------------------------------- Allocation
    CommandSpec::done(
        ALLOCATION,
        "propose",
        K::Create,
        "allocation.propose",
        &[
            reference("person", PERSON).req(),
            reference("subject", TASK).req(),
            when("from").req(),
            when("to").req(),
            text("minutes").req(),
        ],
        &[reference("id", ALLOCATION).req()],
    ),
    CommandSpec::done(
        ALLOCATION,
        "find",
        K::Find,
        "capacity.read",
        &[choice("scope", WHO_ALLOC), when("from"), when("to"), CURSOR],
        &[found("items", ALLOCATION), NEXT],
    ),
    CommandSpec::done(
        ALLOCATION,
        "accept",
        K::Other,
        "allocation.answer",
        &[id(ALLOCATION)],
        &[],
    ),
    CommandSpec::done(
        ALLOCATION,
        "decline",
        K::Other,
        "allocation.answer",
        &[id(ALLOCATION)],
        &[],
    ),
    CommandSpec::done(
        ALLOCATION,
        "withdraw",
        K::Other,
        "allocation.propose",
        &[id(ALLOCATION)],
        &[],
    ),
];
