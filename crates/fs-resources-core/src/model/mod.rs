//! Value types of the resources contract.
//!
//! Pattern: Value objects — plain data, one file per concept; no storage logic.

#[macro_use]
mod names;
mod capacity;
mod need;
mod page;
mod record;
mod reservation;
mod resource;

pub use capacity::{Allocation, AllocationState, Capacity, GroupLoad, GroupShare, Load};
pub use need::{Need, NeedState, Offer, OfferState};
pub use page::{Cursor, Page};
pub use record::{NS, Record, document, from_document};
pub use reservation::{FreePeriod, Reservation, ReservationState};
pub use resource::{Resource, ResourceFilter, ResourceKind};
