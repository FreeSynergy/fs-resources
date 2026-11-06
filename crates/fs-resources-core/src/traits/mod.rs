//! The contract as Rust traits: items, reservations, capacities, needs, feed, group lookup.
//!
//! Pattern: Facade (declarations) — callers depend on these traits, never on `ResourceRepo`.

mod capacities;
mod feed;
mod groups;
mod items;
mod needs;
mod reservations;

pub use capacities::Capacities;
pub use feed::{
    EntityKind, ResourceChange, ResourceEntity, ResourceFeed, ResourceListener, Subscription,
};
pub use groups::GroupLookup;
pub use items::Items;
pub use needs::Needs;
pub use reservations::Reservations;
