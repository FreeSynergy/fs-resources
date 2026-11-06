//! The load of a group: rounded totals of the people who share, hidden below the minimum.
//!
//! Pattern: Specification (use) — only people who switched sharing on for this exact group *and*
//! are members according to the [`GroupLookup`](crate::GroupLookup) port count. Totals are rounded
//! to the nearest 60 minutes times the number of people, so one member's small change disappears.
//! Stated limit: someone who fills a group with accounts of their own, and lets them share, still
//! gets close to one person's hours; each person has to agree to share with that group.

use std::collections::BTreeSet;

use fs_store::{ObjectId, Query, Timestamp};

use super::consts::MIN_SHARING;
use super::handle::ResourceRepo;
use super::reservation_ops::check_range;
use crate::error::Result;
use crate::model::{GroupLoad, GroupShare, Load};
use crate::sweep::round_to;
use crate::unsupported::not_possible;

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
        check_range(from, to)?;
        let q = Query::new().with_index("group", group.to_string());
        let sharing: BTreeSet<String> = self
            .find::<GroupShare>(q)
            .await?
            .into_iter()
            .map(|s| s.person)
            .collect();
        let eligible: BTreeSet<String> = port
            .members(group)
            .await?
            .into_iter()
            .filter(|m| sharing.contains(m))
            .collect();
        if eligible.len() < MIN_SHARING {
            return Ok(GroupLoad {
                hidden: true,
                people: 0,
                load: Load::default(),
            });
        }
        let mut total = Load::default();
        for person in &eligible {
            let one = self.load_of(person, from, to).await?;
            total.capacity_minutes += one.capacity_minutes;
            total.allocated_minutes += one.allocated_minutes;
        }
        let people = eligible.len();
        let step = 60 * u64::try_from(people).unwrap_or(u64::MAX);
        Ok(GroupLoad {
            hidden: false,
            people: u32::try_from(people).unwrap_or(u32::MAX),
            load: Load {
                capacity_minutes: round_to(total.capacity_minutes, step),
                allocated_minutes: round_to(total.allocated_minutes, step),
            },
        })
    }
}
