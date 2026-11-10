//! The one table of commands: every command of the contract and every command a requirement asks for.
//!
//! Pattern: Registry as data — one `const` table, the single source of truth. The type descriptions
//! (`build.rs`), the list of commands the backend must refuse ([`unsupported_commands`]) and the test
//! against `requirements.md` all read this table and nothing else, so a command cannot be in one
//! place and missing in another. A requirement the core does not meet yet is a row with
//! `supported = false`; it is never left out. When the core learns it, the row changes, nothing else.
//!
//! A `change` has no inputs of its own: its input is an `Edit` (field path, basic type, values; missing = unchanged,
//! empty = cleared) over the fields of the type, and the record's id travels next to it.
//! Lists are `find` with paging; a command on one record has a reference to the type itself as input
//! (strategy decision of 2026-11-10). `change` is partial: a missing field stays, an empty one clears.

use std::sync::LazyLock;

use super::commands_needs::NEED_COMMANDS;
use super::commands_reservation::RESERVATION_COMMANDS;
use super::commands_resource::RESOURCE_COMMANDS;
use super::commands_time::TIME_COMMANDS;
use super::spec::{CommandSpec, FieldSpec, reference, text};

/// The requirement rows of `requirements.md` and how the core stands to each (see `requirements.rs`).
pub(super) const R_FORMS: &str = "Generate list and form views from a data type description";
pub(super) const R_LINK: &str = "Link records of one area to another by interface, not by copy";
pub(super) const R_STOCK: &str =
    "Stock of consumables with minimum amount and due date, missing items go to a list";
pub(super) const R_CHECK: &str =
    "Check-out and check-in of an item with status labels deciding assignability";
pub(super) const R_CLASS: &str =
    "Item classes with different rules: tagged, interchangeable, part of another item, used up";

/// The requirement rows that are no command of a data type (the generated frontend meets them).
pub const FRONTEND_REQUIREMENTS: &[&str] = &[R_FORMS];
/// The requirement rows a command stands for.
pub const COMMAND_REQUIREMENTS: &[&str] = &[R_LINK, R_STOCK, R_CHECK, R_CLASS];

pub(super) const CURSOR: FieldSpec = text("cursor");
pub(super) const NEXT: FieldSpec = text("next");

pub(super) const fn id(of: &'static str) -> FieldSpec {
    reference("id", of).req()
}

pub(super) const fn found(name: &'static str, of: &'static str) -> FieldSpec {
    reference(name, of).many()
}

pub(super) const SCOPE_NEEDS: &[&str] = &["all", "open"];
pub(super) const WHO_ALLOC: &[&str] = &["mine", "proposed"];
pub(super) const ON_OFF: &[&str] = &["on", "off"];
pub(super) const CLASSES: &[&str] = &["tagged", "interchangeable", "part", "consumable"];

/// Every command of every type, in the order of the types.
pub static COMMANDS: LazyLock<Vec<CommandSpec>> = LazyLock::new(|| {
    [
        RESOURCE_COMMANDS,
        RESERVATION_COMMANDS,
        NEED_COMMANDS,
        TIME_COMMANDS,
    ]
    .concat()
});

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
