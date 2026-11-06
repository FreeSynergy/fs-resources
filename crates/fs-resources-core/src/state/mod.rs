//! The state machines: one transition function per kind of object.
//!
//! Pattern: State — an enum with a pure function `next(state, step)`; every change of state in the
//! repository goes through it, so an illegal step is refused in one place.

mod allocation;
mod need;
mod reservation;

pub use allocation::{AllocationStep, allocation_next};
pub use need::{OfferStep, close_need, offer_next, settle};
pub use reservation::{Step, reservation_next};
