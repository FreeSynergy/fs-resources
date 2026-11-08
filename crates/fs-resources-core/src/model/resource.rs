//! Things, rooms and material.
//!
//! Pattern: Value object — plain data. `quantity` is how many exist now; for material it is the
//! stock, which only `finish` lowers.

use fs_objects::ObjectId;
use serde::{Deserialize, Serialize};

use super::record::Record;

named_enum! {
    /// What kind of resource it is; the kind decides how a reservation ends.
    ResourceKind {
        /// Lent and given back: a projector, a tent.
        Thing = "thing",
        /// A thing in one place; usually a quantity of 1. The rules are those of a thing.
        Room = "room",
        /// Used up: taking it lowers the stock for good.
        Material = "material",
    }
}

/// A resource.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Resource {
    /// Id (UUIDv7).
    pub id: ObjectId,
    /// Display name (also the envelope name).
    pub name: String,
    /// Thing, room or material.
    pub kind: ResourceKind,
    /// How many exist now.
    pub quantity: u32,
    /// Free text: piece, litre, ...
    pub unit: String,
    /// Person or group IRI; empty = none.
    pub owner: String,
    /// Place IRI; empty = none.
    pub place: String,
    /// Free labels.
    pub labels: Vec<String>,
    /// Internal: set while a delete is under way. Such a resource is hidden and takes no
    /// reservations; deleting again resumes.
    #[serde(default)]
    pub deleting: bool,
}

/// Which resources `list` returns; every field that is set must match.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ResourceFilter {
    /// Only this kind.
    pub kind: Option<ResourceKind>,
    /// Only this owner; empty = any.
    pub owner: String,
    /// Only this place; empty = any.
    pub place: String,
    /// Only resources with this label; empty = any.
    pub label: String,
    /// Part of the name, any case; empty = any.
    pub name: String,
}

impl Record for Resource {
    const TYPE: &'static str = "Resource";
    fn id(&self) -> ObjectId {
        self.id
    }
    fn name(&self) -> String {
        self.name.clone()
    }
    fn index(&self) -> Vec<(&'static str, String)> {
        let mut ix = vec![
            ("kind", self.kind.as_str().into()),
            ("owner", self.owner.clone()),
            ("place", self.place.clone()),
        ];
        ix.extend(self.labels.iter().map(|l| ("label", l.clone())));
        ix
    }
}
