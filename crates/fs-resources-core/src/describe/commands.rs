//! The one table of commands: every command of the contract and every command a requirement asks for.
//!
//! Pattern: Registry as data — one `const` table, the single source of truth. The type descriptions
//! (`build.rs`), the list of commands the backend must refuse ([`unsupported_commands`]) and the test
//! against `requirements.md` all read this table and nothing else, so a command cannot be in one
//! place and missing in another. A requirement the core does not meet yet is a row with
//! `supported = false`; it is never left out. When the core learns it, the row changes, nothing else.
//!
//! Lists are `find` with paging; a command on one record has a reference to the type itself as input
//! (strategy decision of 2026-11-10). `change` is partial: a missing field stays, an empty one clears.

use fs_types::BasicType;
use fs_types::CommandKind as K;

use super::fields::{ALLOCATION, CAPACITY, KINDS, NEED, OFFER, RESERVATION, RESOURCE};
use super::spec::{CommandSpec, FieldSpec, choice, reference, text, when};

/// The requirement rows of `requirements.md` and how the core stands to each (see `requirements.rs`).
const R_FORMS: &str = "Generate list and form views from a data type description";
const R_LINK: &str = "Link records of one area to another by interface, not by copy";
const R_STOCK: &str =
    "Stock of consumables with minimum amount and due date, missing items go to a list";
const R_CHECK: &str = "Check-out and check-in of an item with status labels deciding assignability";
const R_CLASS: &str =
    "Item classes with different rules: tagged, interchangeable, part of another item, used up";

/// The requirement rows that are no command of a data type (the generated frontend meets them).
pub const FRONTEND_REQUIREMENTS: &[&str] = &[R_FORMS];
/// The requirement rows a command stands for.
pub const COMMAND_REQUIREMENTS: &[&str] = &[R_LINK, R_STOCK, R_CHECK, R_CLASS];

const CURSOR: FieldSpec = text("cursor");
const NEXT: FieldSpec = text("next");

const fn id(of: &'static str) -> FieldSpec {
    reference("id", of).req()
}

const fn found(name: &'static str, of: &'static str) -> FieldSpec {
    reference(name, of).many()
}

const SCOPE_NEEDS: &[&str] = &["all", "open"];
const WHO_ALLOC: &[&str] = &["mine", "proposed"];
const ON_OFF: &[&str] = &["on", "off"];
const CLASSES: &[&str] = &["tagged", "interchangeable", "part", "consumable"];

/// Every command of every type.
pub const COMMANDS: &[CommandSpec] = &[
    // ---------------------------------------------------------------- Resource
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
            reference("owner", ""),
            reference("place", ""),
        ],
        &[reference("id", RESOURCE).req()],
    ),
    CommandSpec::done(
        RESOURCE,
        "change",
        K::Change,
        "resource.edit",
        &[
            id(RESOURCE),
            text("name"),
            text("unit"),
            reference("owner", ""),
            reference("place", ""),
            FieldSpec::new("labels", BasicType::Keywords),
            text("quantity"),
        ],
        &[],
    ),
    CommandSpec::done(
        RESOURCE,
        "find",
        K::Find,
        "resource.read",
        &[
            text("term"),
            choice("kind", KINDS),
            reference("owner", ""),
            reference("place", ""),
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
        K::Change,
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
        &[id(RESOURCE), reference("holder", "").req(), when("to")],
        &[reference("reservation", RESERVATION)],
    ),
    CommandSpec::missing(RESOURCE, "checkIn", K::Other, R_CHECK, &[id(RESOURCE)], &[]),
    CommandSpec::missing(
        RESOURCE,
        "setStatus",
        K::Change,
        R_CHECK,
        &[id(RESOURCE), text("status").req()],
        &[],
    ),
    CommandSpec::missing(
        RESOURCE,
        "setClass",
        K::Change,
        R_CLASS,
        &[id(RESOURCE), choice("class", CLASSES).req()],
        &[],
    ),
    CommandSpec::missing(
        RESOURCE,
        "link",
        K::Other,
        R_LINK,
        &[id(RESOURCE), reference("target", "").req()],
        &[],
    ),
    // ---------------------------------------------------------------- Reservation
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
            reference("holder", "").req(),
            reference("subject", ""),
            text("note"),
        ],
        &[reference("id", RESERVATION).req()],
    ),
    CommandSpec::done(
        RESERVATION,
        "change",
        K::Change,
        "reservation.request",
        &[id(RESERVATION), text("quantity"), when("from"), when("to")],
        &[],
    ),
    CommandSpec::done(
        RESERVATION,
        "find",
        K::Find,
        "reservation.read",
        &[
            reference("resource", RESOURCE),
            reference("holder", ""),
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
    // ---------------------------------------------------------------- Need
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
            reference("subject", ""),
        ],
        &[reference("id", NEED).req()],
    ),
    CommandSpec::done(
        NEED,
        "change",
        K::Change,
        "need.create",
        &[
            id(NEED),
            text("what"),
            text("quantity"),
            text("unit"),
            when("from"),
            when("to"),
        ],
        &[],
    ),
    CommandSpec::done(
        NEED,
        "find",
        K::Find,
        "need.read",
        &[
            reference("subject", ""),
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
            reference("by", "").req(),
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
    // ---------------------------------------------------------------- Capacity (the calling person's own)
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
    CommandSpec::done(
        CAPACITY,
        "change",
        K::Change,
        "capacity.set",
        &[
            id(CAPACITY),
            when("from"),
            when("to"),
            text("minutesPerWeek"),
            text("note"),
        ],
        &[],
    ),
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
            reference("person", "").req(),
            reference("subject", "").req(),
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

/// The commands of one data type, in table order.
pub fn commands_of(data_type: &str) -> impl Iterator<Item = &'static CommandSpec> + '_ {
    COMMANDS.iter().filter(move |c| c.of == data_type)
}

/// One command of one type.
pub fn command(data_type: &str, name: &str) -> Option<&'static CommandSpec> {
    commands_of(data_type).find(|c| c.name == name)
}

/// The commands the core cannot carry out (`supported = false`): the wire has no flag yet, so a
/// backend refuses these by name.
pub fn unsupported_commands() -> impl Iterator<Item = &'static CommandSpec> {
    COMMANDS.iter().filter(|c| !c.supported)
}

/// Whether `data_type`'s command `name` is in the table but not supported.
pub fn is_unsupported(data_type: &str, name: &str) -> bool {
    command(data_type, name).is_some_and(|c| !c.supported)
}
