//! The load of a group: rounded weekly totals, hidden unless enough members contribute.
//!
//! Pattern: Specification (use) — the caller must be a member; only members who switched sharing on
//! for this exact group *and* have non-zero capacity in the week contribute to it. The range is
//! whole weeks on a fixed grid: `from` must be a Monday 00:00 UTC, so no search can slide it. Each
//! week is summed and rounded on its own to [`GROUP_ROUND`] minutes. A week with fewer than
//! [`MIN_SHARING`] contributors, or a caller who is no member, makes the whole answer `hidden`;
//! a week with fewer than that many people with allocations sets `allocations_hidden` and leaves
//! `allocated_minutes` at 0. `people` is always 0. Stated limits: a one-minute change at a rounding
//! boundary moves a week by a whole step; and someone who fills a group with accounts of their own,
//! who agree to share, still gets close to one person's hours.

use std::collections::BTreeSet;

use chrono::{Datelike, NaiveTime, Weekday};
use fs_store::{ObjectId, Query, Timestamp};

use super::consts::{GROUP_ROUND, MAX_WEEKS, MIN_SHARING};
use super::handle::ResourceRepo;
use super::load::Records;
use crate::error::{Error, Result};
use crate::model::{GroupLoad, GroupShare, Load};
use crate::sweep::round_to;
use crate::unsupported::not_possible;

const HIDDEN: GroupLoad = GroupLoad {
    hidden: true,
    people: 0,
    allocations_hidden: false,
    load: Load {
        capacity_minutes: 0,
        allocated_minutes: 0,
    },
};

const BAD_RANGE: Error =
    Error::Invalid("the range must be whole weeks from a Monday 00:00 UTC, at most 520");

/// The weeks `[from, to)` is made of, or the refusal. Never panics, whatever the times.
fn weeks(from: Timestamp, to: Timestamp) -> Result<Vec<(Timestamp, Timestamp)>> {
    if from.weekday() != Weekday::Mon || from.time() != NaiveTime::MIN || to <= from {
        return Err(BAD_RANGE);
    }
    let week = chrono::Duration::weeks(1);
    let mut out = Vec::new();
    let mut start = from;
    while start < to {
        let end = start.checked_add_signed(week).ok_or(BAD_RANGE)?;
        if end > to || out.len() >= MAX_WEEKS {
            return Err(BAD_RANGE);
        }
        out.push((start, end));
        start = end;
    }
    Ok(out)
}

impl ResourceRepo {
    pub(super) async fn group_total(
        &self,
        group: ObjectId,
        from: Timestamp,
        to: Timestamp,
    ) -> Result<GroupLoad> {
        let Some(port) = &self.groups else {
            return not_possible("capacity.group");
        };
        let weeks = weeks(from, to)?;
        let members: BTreeSet<String> = port.members(group).await?.into_iter().collect();
        if !members.contains(&self.actor) {
            return Ok(HIDDEN);
        }
        let q = Query::new().with_index("group", group.to_string());
        // each person counts once, however many share records they have
        let sharing: BTreeSet<String> = self
            .find::<GroupShare>(q)
            .await?
            .into_iter()
            .map(|s| s.person)
            .filter(|p| members.contains(p))
            .collect();
        let mut records: Vec<Records> = Vec::new();
        for person in &sharing {
            records.push(self.records_of(person).await?);
        }
        let (mut total, mut allocations_hidden) = (Load::default(), false);
        for (start, end) in weeks {
            let each: Vec<Load> = records
                .iter()
                .map(|r| r.load(start, end))
                .filter(|l| l.capacity_minutes > 0)
                .collect();
            if each.len() < MIN_SHARING {
                return Ok(HIDDEN);
            }
            let with_allocations: Vec<_> =
                each.iter().filter(|l| l.allocated_minutes > 0).collect();
            allocations_hidden |= with_allocations.len() < MIN_SHARING;
            let cap: u64 = each.iter().map(|l| l.capacity_minutes).sum();
            let alloc: u64 = with_allocations.iter().map(|l| l.allocated_minutes).sum();
            total.capacity_minutes += round_to(cap, GROUP_ROUND);
            total.allocated_minutes += round_to(alloc, GROUP_ROUND);
        }
        if allocations_hidden {
            total.allocated_minutes = 0;
        }
        Ok(GroupLoad {
            hidden: false,
            people: 0,
            allocations_hidden,
            load: total,
        })
    }
}
