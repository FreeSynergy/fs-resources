//! Minutes pro rata by overlap, and rounding.
//!
//! Pattern: Specification — integer arithmetic only (`i128` microseconds), rounding always down
//! except `round_to`, which rounds to the nearest step with half rounding up.

use fs_objects::Timestamp;

use crate::model::{Allocation, AllocationState, Capacity};

const WEEK_MICROS: i128 = 7 * 24 * 3600 * 1_000_000;

fn micros(from: Timestamp, to: Timestamp) -> i128 {
    i128::from(to.timestamp_micros()) - i128::from(from.timestamp_micros())
}

/// Whether `[a, b)` and `[c, d)` share any time; touching ranges do not.
pub fn overlaps(a: Timestamp, b: Timestamp, c: Timestamp, d: Timestamp) -> bool {
    a < d && c < b
}

fn overlap_micros(a: Timestamp, b: Timestamp, c: Timestamp, d: Timestamp) -> i128 {
    if overlaps(a, b, c, d) {
        micros(a.max(c), b.min(d))
    } else {
        0
    }
}

/// Capacity in `[from, to)`: minutes per week pro rata by overlap, summed exactly and divided once,
/// so the result is rounded down once.
pub fn capacity_minutes(items: &[Capacity], from: Timestamp, to: Timestamp) -> u64 {
    let sum: i128 = items
        .iter()
        .map(|c| i128::from(c.minutes_per_week) * overlap_micros(c.from, c.to, from, to))
        .sum();
    u64::try_from(sum / WEEK_MICROS).unwrap_or(0)
}

/// Accepted allocations in `[from, to)`: each one's minutes pro rata by overlap with its own range,
/// rounded down per allocation, then summed.
pub fn allocated_minutes(items: &[Allocation], from: Timestamp, to: Timestamp) -> u64 {
    items
        .iter()
        .filter(|a| a.state == AllocationState::Accepted)
        .map(|a| {
            let part = i128::from(a.minutes) * overlap_micros(a.from, a.to, from, to);
            u64::try_from(part / micros(a.from, a.to).max(1)).unwrap_or(0)
        })
        .sum()
}

/// `total` rounded to the nearest multiple of `step` (half rounds up).
pub fn round_to(total: u64, step: u64) -> u64 {
    (total + step / 2) / step * step
}
