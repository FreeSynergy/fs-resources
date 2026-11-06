//! A person's time: capacity, allocations, group shares, load.
//!
//! Pattern: Value object — plain data. All three stored kinds are private ([`Record::PRIVATE`]).

use fs_store::{ObjectId, Timestamp};
use serde::{Deserialize, Serialize};

use super::record::Record;

/// Someone's own statement: minutes a week, in a range `[from, to)`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Capacity {
    /// Id (UUIDv7).
    pub id: ObjectId,
    /// Set by the provider: always the calling person.
    pub person: String,
    /// Start (included).
    pub from: Timestamp,
    /// End (excluded).
    pub to: Timestamp,
    /// Focus minutes per week.
    pub minutes_per_week: u32,
    /// Seen only by that person.
    pub note: String,
}

named_enum! {
    /// Where an allocation stands.
    AllocationState {
        /// Asked for, no answer yet.
        Proposed = "proposed",
        /// The person said yes; only this counts in a load.
        Accepted = "accepted",
        /// The person said no.
        Declined = "declined",
        /// Taken back by who asked.
        Withdrawn = "withdrawn",
    }
}

/// A request for someone's hours; counts only once they accept.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Allocation {
    /// Id (UUIDv7).
    pub id: ObjectId,
    /// Whose hours.
    pub person: String,
    /// IRI of the task.
    pub subject: String,
    /// Start (included).
    pub from: Timestamp,
    /// End (excluded).
    pub to: Timestamp,
    /// In total, over the range.
    pub minutes: u32,
    /// Where it stands.
    pub state: AllocationState,
    /// Who asked.
    pub proposer: String,
}

/// Minutes available and minutes taken by accepted allocations.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Load {
    /// Capacity in the range.
    pub capacity_minutes: u64,
    /// Accepted allocations in the range.
    pub allocated_minutes: u64,
}

/// What a group may see of its members' time.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct GroupLoad {
    /// Too few people share; everything else stays empty.
    pub hidden: bool,
    /// Unused, always 0: a count would show who is absent in a week.
    pub people: u32,
    /// Too few people have allocations in some week; `allocated_minutes` stays 0.
    pub allocations_hidden: bool,
    /// Rounded totals.
    pub load: Load,
}

/// A person's decision to share their load with one group.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GroupShare {
    /// Id (UUIDv7).
    pub id: ObjectId,
    /// Who shares.
    pub person: String,
    /// With which group.
    pub group: ObjectId,
}

impl Record for Capacity {
    const TYPE: &'static str = "Capacity";
    const PRIVATE: bool = true;
    fn id(&self) -> ObjectId {
        self.id
    }
    fn name(&self) -> String {
        "capacity".into()
    }
    fn index(&self) -> Vec<(&'static str, String)> {
        vec![("person", self.person.clone())]
    }
}

impl Record for Allocation {
    const TYPE: &'static str = "Allocation";
    const PRIVATE: bool = true;
    fn id(&self) -> ObjectId {
        self.id
    }
    fn name(&self) -> String {
        "allocation".into()
    }
    fn index(&self) -> Vec<(&'static str, String)> {
        vec![
            ("person", self.person.clone()),
            ("proposer", self.proposer.clone()),
            ("state", self.state.as_str().into()),
        ]
    }
}

impl Record for GroupShare {
    const TYPE: &'static str = "GroupShare";
    const PRIVATE: bool = true;
    fn id(&self) -> ObjectId {
        self.id
    }
    fn name(&self) -> String {
        "share".into()
    }
    fn index(&self) -> Vec<(&'static str, String)> {
        vec![
            ("person", self.person.clone()),
            ("group", self.group.to_string()),
        ]
    }
}
