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
    reference("owner", ""),
    reference("place", ""),
    FieldSpec::new("labels", BasicType::Keywords),
];

/// Fields of a reservation.
pub const RESERVATION_FIELDS: &[FieldSpec] = &[
    reference("resource", RESOURCE).req(),
    text("quantity").req(),
    when("from").req(),
    when("to").req(),
    reference("holder", ""),
    choice("state", RESERVATION_STATES),
    reference("subject", ""),
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
    reference("subject", ""),
    choice("state", NEED_STATES),
    text("covered"),
];

/// Fields of an offer.
pub const OFFER_FIELDS: &[FieldSpec] = &[
    reference("need", NEED).req(),
    reference("by", "").req(),
    text("quantity").req(),
    reference("resource", RESOURCE),
    choice("state", OFFER_STATES),
    text("note"),
    reference("reservation", RESERVATION),
];

/// Fields of a capacity.
pub const CAPACITY_FIELDS: &[FieldSpec] = &[
    reference("person", ""),
    when("from").req(),
    when("to").req(),
    text("minutesPerWeek").req(),
    text("note"),
];

/// Fields of an allocation.
pub const ALLOCATION_FIELDS: &[FieldSpec] = &[
    reference("person", "").req(),
    reference("subject", "").req(),
    when("from").req(),
    when("to").req(),
    text("minutes").req(),
    choice("state", ALLOCATION_STATES),
    reference("proposer", ""),
];
