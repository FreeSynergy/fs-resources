//! Free quantity over time.
//!
//! Pattern: Specification (sweep) — cut the asked range at every start and end of a hold, count
//! what is held in each piece, and merge neighbours that have the same free quantity.

use fs_store::Timestamp;

use super::hold::Hold;
use crate::model::FreePeriod;

fn held_at(holds: &[Hold], t: Timestamp) -> u64 {
    holds
        .iter()
        .filter(|h| h.covers(t))
        .map(|h| u64::from(h.quantity))
        .sum()
}

/// Stretches of equal free quantity over `[from, to)`, neighbours with the same quantity merged.
/// The caller makes sure `to > from`.
pub fn free_periods(stock: u32, holds: &[Hold], from: Timestamp, to: Timestamp) -> Vec<FreePeriod> {
    let mut cuts = vec![from, to];
    cuts.extend(
        holds
            .iter()
            .flat_map(|h| [h.from, h.to])
            .filter(|t| *t > from && *t < to),
    );
    cuts.sort();
    cuts.dedup();
    let mut out: Vec<FreePeriod> = Vec::new();
    for w in cuts.windows(2) {
        let free = u64::from(stock).saturating_sub(held_at(holds, w[0]));
        let free = u32::try_from(free).unwrap_or(stock);
        match out.last_mut() {
            Some(last) if last.free == free => last.to = w[1],
            _ => out.push(FreePeriod {
                from: w[0],
                to: w[1],
                free,
            }),
        }
    }
    out
}

/// The lowest free quantity anywhere in `[from, to)`: what a new hold there can have.
pub fn lowest_free(stock: u32, holds: &[Hold], from: Timestamp, to: Timestamp) -> u32 {
    free_periods(stock, holds, from, to)
        .iter()
        .map(|p| p.free)
        .min()
        .unwrap_or(stock)
}

/// The most that is held at any one time.
pub fn peak(holds: &[Hold]) -> u64 {
    holds
        .iter()
        .map(|h| held_at(holds, h.from))
        .max()
        .unwrap_or(0)
}
