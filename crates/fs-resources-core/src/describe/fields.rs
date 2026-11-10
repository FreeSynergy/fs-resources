//! The fields of each data type, as data.
//!
//! Pattern: Value object tables — one `const` list per type, in the order a form shows them. The
//! standard of each type is named in `build.rs`; there is none for a resource model, so these follow
//! `resources.capnp` (DESIGN.md, "Standards").

use super::spec::{FieldSpec, choice, reference, text, when};
use fs_types::BasicType;

/// Names of the data types, as in the catalog.
pub const RESOURCE: &str = "Resource";
/// A reservation.
pub const RESERVATION: &str = "Reservation";
/// A need.
pub const NEED: &str = "Need";
/// An offer.
pub const OFFER: &str = "Offer";
/// A person's capacity.
pub const CAPACITY: &str = "Capacity";
/// An allocation of a person's hours.
pub const ALLOCATION: &str = "Allocation";

/// Data types of other programs that these fields refer to (ADR 0050). A person field also takes a group IRI and
/// a subject also an event or project IRI: a reference names one type, the rule of `fs-types` only checks the IRI.
pub const PERSON: &str = "Person";
/// A place (fs-maps).
pub const PLACE: &str = "Place";
/// A task (fs-tasks).
pub const TASK: &str = "Task";
/// Anything (for a link to a record of another program): the rule for "any" is the empty target, which makes the
/// command a record command; `link` is one on purpose.
pub const ANY: &str = "";

/// The kinds of resource.
pub const KINDS: &[&str] = &["thing", "room", "material"];
/// States of a reservation.
pub const RESERVATION_STATES: &[&str] = &[
    "requested",
    "confirmed",
    "returned",
    "cancelled",
    "declined",
];
/// States of a need.
pub const NEED_STATES: &[&str] = &["open", "covered", "closed"];
/// States of an offer.
pub const OFFER_STATES: &[&str] = &["offered", "accepted", "declined", "withdrawn"];
/// States of an allocation.
pub const ALLOCATION_STATES: &[&str] = &["proposed", "accepted", "declined", "withdrawn"];

/// Fields of a resource.
pub const RESOURCE_FIELDS: &[FieldSpec] = &[
    text("name").req(),
    choice("kind", KINDS).req(),
    text("quantity").req(),
    text("unit"),
    reference("owner", PERSON),
    reference("place", PLACE),
    FieldSpec::new("labels", BasicType::Keywords),
];

/// Fields of a reservation.
pub const RESERVATION_FIELDS: &[FieldSpec] = &[
    reference("resource", RESOURCE).req(),
    text("quantity").req(),
    when("from").req(),
    when("to").req(),
    reference("holder", PERSON),
    choice("state", RESERVATION_STATES),
    reference("subject", TASK),
    text("note"),
    text("done"),
];

/// Fields of a need.
pub const NEED_FIELDS: &[FieldSpec] = &[
    text("what").req(),
    text("quantity").req(),
    text("unit"),
    when("from"),
    when("to"),
    reference("subject", TASK),
    choice("state", NEED_STATES),
    text("covered"),
];

/// Fields of an offer.
pub const OFFER_FIELDS: &[FieldSpec] = &[
    reference("need", NEED).req(),
    reference("by", PERSON).req(),
    text("quantity").req(),
    reference("resource", RESOURCE),
    choice("state", OFFER_STATES),
    text("note"),
    reference("reservation", RESERVATION),
];

/// Fields of a capacity.
pub const CAPACITY_FIELDS: &[FieldSpec] = &[
    reference("person", PERSON),
    when("from").req(),
    when("to").req(),
    text("minutesPerWeek").req(),
    text("note"),
];

/// Fields of an allocation.
pub const ALLOCATION_FIELDS: &[FieldSpec] = &[
    reference("person", PERSON).req(),
    reference("subject", TASK).req(),
    when("from").req(),
    when("to").req(),
    text("minutes").req(),
    choice("state", ALLOCATION_STATES),
    reference("proposer", PERSON),
];
