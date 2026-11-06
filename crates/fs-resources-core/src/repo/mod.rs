//! The store-backed resources program: module map.
//!
//! Pattern: Facade (declarations) — `ResourceRepo` is the one entry point; each file is one
//! concept (storage, items, bookings, reservations, capacity, group load, needs, feed).

mod allocations;
mod booking;
mod capacity;
mod change;
mod consts;
mod feed;
mod finish;
mod group_load;
mod handle;
mod io;
mod items;
mod load;
mod needs_ops;
mod offers;
mod provide_capacity;
mod provide_needs;
mod provide_reservations;
mod reservation_ops;
mod stock;
mod support;

pub use consts::{CONTEXT, GROUP_ROUND, MIN_SHARING, WRITABLE_TYPES};
pub use handle::ResourceRepo;
