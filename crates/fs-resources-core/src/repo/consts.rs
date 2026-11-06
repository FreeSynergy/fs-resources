//! Constants of the resources objects.
//!
//! Pattern: Registry (constants) — context, media type, the types the hub may be told about, and
//! the fixed numbers of the rules, in one place.

use fs_store::Timestamp;

/// `context` of all objects of this program.
pub const CONTEXT: &str = "resources";

/// The store types this program writes with the activity log. The hub manifest may list exactly
/// these in `writes`. Capacity, allocations and shares are not here: they are written without the
/// log and must never reach the hub.
pub const WRITABLE_TYPES: [&str; 4] = ["Resource", "Reservation", "Need", "Offer"];

/// The fewest people who must share with a group before it sees anything of their time.
pub const MIN_SHARING: usize = 5;

/// A group total is rounded to the nearest multiple of this many minutes, one week at a time.
pub const GROUP_ROUND: u64 = 300;

/// The most weeks one `group_load` may ask for.
pub(super) const MAX_WEEKS: usize = 520;

pub(super) const AS2_TYPE: &str = "application/activity+json";

/// Stored as the time of every private record, so no write time is kept.
pub(super) const EPOCH: Timestamp = Timestamp::UNIX_EPOCH;
