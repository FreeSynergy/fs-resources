//! The load of a group: rounded weekly totals, hidden unless enough members contribute.
//!
//! Pattern: Specification (use) — the caller must be a member; only members who switched sharing on
//! for this exact group *and* have non-zero capacity in the week contribute to it. The range is
//! whole weeks from `from`; each week is summed and rounded on its own to [`GROUP_ROUND`] minutes,
//! so a long range cannot be used to read a rate. A week with fewer than [`MIN_SHARING`]
//! contributors, or a caller who is no member, makes the whole answer `hidden`. Stated limits: a
//! one-minute change at a rounding boundary moves a week by a whole step; and someone who fills a
//! group with accounts of their own, who agree to share, still gets close to one person's hours.

use std::collections::BTreeSet;

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
    load: Load {
        capacity_minutes: 0,
        allocated_minutes: 0,
    },
};

/// The weeks `[from, to)` is made of, or the refusal.
fn weeks(from: Timestamp, to: Timestamp) -> Result<Vec<(Timestamp, Timestamp)>> {
    let week = chrono::Duration::weeks(1);
    let mut out = Vec::new();
    let mut start = from;
    while start < to {
        let end = start + week;
        if end > to || out.len() >= MAX_WEEKS {
            return Err(Error::Invalid("the range must be whole weeks, at most 520"));
        }
        out.push((start, end));
        start = end;
    }
    if out.is_empty() {
        return Err(Error::Invalid("`to` must be after `from`"));
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
        let sharing = self.find::<GroupShare>(q).await?;
        let mut records: Vec<Records> = Vec::new();
        for s in sharing.iter().filter(|s| members.contains(&s.person)) {
            records.push(self.records_of(&s.person).await?);
        }
        let (mut total, mut fewest) = (Load::default(), usize::MAX);
        for (start, end) in weeks {
            let each: Vec<Load> = records
                .iter()
                .map(|r| r.load(start, end))
                .filter(|l| l.capacity_minutes > 0)
                .collect();
            if each.len() < MIN_SHARING {
                return Ok(HIDDEN);
            }
            fewest = fewest.min(each.len());
            let cap: u64 = each.iter().map(|l| l.capacity_minutes).sum();
            let alloc: u64 = each.iter().map(|l| l.allocated_minutes).sum();
            total.capacity_minutes += round_to(cap, GROUP_ROUND);
            total.allocated_minutes += round_to(alloc, GROUP_ROUND);
        }
        Ok(GroupLoad {
            hidden: false,
            people: u32::try_from(fewest).unwrap_or(u32::MAX),
            load: total,
        })
    }
}
