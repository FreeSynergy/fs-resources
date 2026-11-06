//! The pure arithmetic of the program: what is held over time, free periods, pro rata minutes.
//!
//! Pattern: Specification — small side-effect-free functions over plain values; every check of a
//! reservation and every load goes through them, and they need no store.

mod free;
mod hold;
mod prorate;

pub use free::{free_periods, lowest_free, peak};
pub use hold::{FOREVER, Hold};
pub use prorate::{allocated_minutes, capacity_minutes, overlaps, round_to};
