//! The port to the groups program.
//!
//! Pattern: Adapter (port) — this core does not depend on `fs-groups-core`; whoever wires the
//! programs together implements this trait over the groups program (effective membership).

use async_trait::async_trait;
use fs_store::ObjectId;

use crate::error::Result;

/// Who is in a group.
#[async_trait]
pub trait GroupLookup: Send + Sync {
    /// The IRIs of the people who are members of `group`, nested groups included (effective
    /// members). A group that does not exist has none.
    async fn members(&self, group: ObjectId) -> Result<Vec<String>>;
}
