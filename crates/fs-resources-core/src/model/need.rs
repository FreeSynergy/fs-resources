//! Needs and offers.
//!
//! Pattern: Value object — plain data; the transitions live in `state`.

use fs_objects::{ObjectId, Timestamp};
use serde::{Deserialize, Serialize};

use super::record::Record;

named_enum! {
    /// Where a need stands.
    NeedState {
        /// Not yet covered.
        Open = "open",
        /// Accepted offers add up to the quantity.
        Covered = "covered",
        /// No longer wanted; takes no offers.
        Closed = "closed",
    }
}

named_enum! {
    /// Where an offer stands.
    OfferState {
        /// Made, not yet decided.
        Offered = "offered",
        /// Taken; counts towards `covered`.
        Accepted = "accepted",
        /// Refused, or taken back after acceptance, by who decides.
        Declined = "declined",
        /// Taken back by who offered.
        Withdrawn = "withdrawn",
    }
}

/// Something a project, group or task needs.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Need {
    /// Id (UUIDv7).
    pub id: ObjectId,
    /// What is needed, in words.
    pub what: String,
    /// How many.
    pub quantity: u32,
    /// Free text: piece, litre, ...
    pub unit: String,
    /// Start (included); `None` = the need has no time.
    pub from: Option<Timestamp>,
    /// End (excluded); set together with `from`.
    pub to: Option<Timestamp>,
    /// Project, group or task IRI.
    pub subject: String,
    /// Where it stands.
    pub state: NeedState,
    /// Sum of the accepted offers.
    pub covered: u32,
}

/// Someone's offer to cover (part of) a need.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Offer {
    /// Id (UUIDv7).
    pub id: ObjectId,
    /// The need.
    pub need: ObjectId,
    /// Person or group IRI.
    pub by: String,
    /// How many.
    pub quantity: u32,
    /// One of our resources, if the offer names one.
    pub resource: Option<ObjectId>,
    /// Where it stands.
    pub state: OfferState,
    /// Free note.
    pub note: String,
    /// Made on accepting, if the offer names a resource and the need has a time.
    pub reservation: Option<ObjectId>,
}

impl Record for Need {
    const TYPE: &'static str = "Need";
    fn id(&self) -> ObjectId {
        self.id
    }
    fn name(&self) -> String {
        self.what.clone()
    }
    fn index(&self) -> Vec<(&'static str, String)> {
        vec![
            ("subject", self.subject.clone()),
            ("state", self.state.as_str().into()),
        ]
    }
}

impl Record for Offer {
    const TYPE: &'static str = "Offer";
    fn id(&self) -> ObjectId {
        self.id
    }
    fn name(&self) -> String {
        format!("Offer by {}", self.by)
    }
    fn index(&self) -> Vec<(&'static str, String)> {
        vec![
            ("need", self.need.to_string()),
            ("by", self.by.clone()),
            ("state", self.state.as_str().into()),
            (
                "reservation",
                self.reservation.map(|r| r.to_string()).unwrap_or_default(),
            ),
        ]
    }
}
