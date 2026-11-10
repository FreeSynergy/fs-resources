//! `fs-resources-core`: program core of FreeSynergy resources (no GUI, no capnp server yet).
//!
//! What we have (things, rooms, material), who has time (capacity), what we need and who offers it.
//! Objects are stored through `fs-objects` with their own members in the `fs:` namespace (ADR 0014,
//! 0020, 0050). A person's capacity, allocations and shares are private: written without the
//! activity log, readable only by that person, and a group sees rounded totals from at least five
//! people who agreed. Start with [`ResourceRepo`] and the traits [`Items`], [`Reservations`],
//! [`Capacities`], [`Needs`] and [`ResourceFeed`]. See `DESIGN.md` for the pattern map.
//!
//! Pattern: Facade — this file only declares modules and re-exports the public API.

#![deny(missing_docs)]
#![warn(clippy::pedantic)]
// Pedantic lints that would only add noise to this crate's small, documented API: every fallible fn
// returns the crate `Result` (documented once on `Error`), results of pure getters are used anyway,
// and the methods of the contract take as many arguments as the schema gives them.
#![allow(
    clippy::missing_errors_doc,
    clippy::must_use_candidate,
    clippy::module_name_repetitions,
    clippy::too_many_arguments
)]

pub mod describe;
mod error;
pub mod model;
mod repo;
pub mod state;
pub mod sweep;
pub mod traits;
mod unsupported;

pub use error::{Error, Result};
pub use model::{
    Allocation, AllocationState, Capacity, Cursor, FreePeriod, GroupLoad, GroupShare, Load, Need,
    NeedState, Offer, OfferState, Page, Reservation, ReservationState, Resource, ResourceFilter,
    ResourceKind,
};
pub use repo::{CONTEXT, GROUP_ROUND, MIN_SHARING, ResourceRepo, WRITABLE_TYPES};
pub use traits::{
    Capacities, EntityKind, GroupLookup, Items, Needs, Reservations, ResourceChange,
    ResourceEntity, ResourceFeed, ResourceListener, Subscription,
};
pub use unsupported::not_possible;
