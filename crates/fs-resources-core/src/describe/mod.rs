//! The type description of the program (ADR 0066): what a generated frontend is told.
//!
//! Pattern: Registry — `commands.rs` is the one table; `build.rs` makes the `fs-types` descriptions
//! from it and `registry()` collects them for the `types` method of the contract.

mod build;
mod commands;
mod fields;
mod spec;

pub use build::{PROGRAM, registry};
pub use commands::{
    COMMAND_REQUIREMENTS, COMMANDS, FRONTEND_REQUIREMENTS, command, commands_of, is_unsupported,
    unsupported_commands,
};
pub use fields::{
    ALLOCATION, ALLOCATION_STATES, CAPACITY, KINDS, NEED, NEED_STATES, OFFER, OFFER_STATES,
    RESERVATION, RESERVATION_STATES, RESOURCE,
};
pub use spec::{CommandSpec, FieldSpec};
