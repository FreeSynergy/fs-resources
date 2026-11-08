//! The [`Items`] contract: things, rooms and material.
//!
//! Pattern: Repository (interface) + Template method + Null Object — callers use `dyn Items`. Like a
//! parent class (ADR 0045): every provider has every method; one it does not override answers an
//! explicit `Unsupported`. The Rust form of `Items` in `resources.capnp`.

use async_trait::async_trait;
use fs_interfaces::Capabilities;
use fs_objects::ObjectId;

use crate::error::Result;
use crate::model::{Cursor, Page, Resource, ResourceFilter, ResourceKind};
use crate::unsupported::not_possible;

/// Resources. Interface status: draft (ADR 0045).
#[async_trait]
pub trait Items: Send + Sync {
    /// What this provider can do, per capability id (for the whole program, not only items).
    fn capabilities(&self) -> Capabilities;
    /// `resource.read`: one resource, if it exists.
    async fn get(&self, id: ObjectId) -> Result<Option<Resource>>;
    /// `resource.read`: resources in name order, one page.
    async fn list(
        &self,
        filter: &ResourceFilter,
        cursor: Option<&Cursor>,
    ) -> Result<Page<Resource>>;

    /// `resource.create`: a new resource; the quantity must not be zero.
    async fn create(
        &self,
        _kind: ResourceKind,
        _name: &str,
        _quantity: u32,
        _unit: &str,
        _owner: &str,
        _place: &str,
    ) -> Result<Resource> {
        not_possible("resource.create")
    }
    /// `resource.edit`: replace name, unit, owner, place and labels.
    async fn edit(
        &self,
        _id: ObjectId,
        _name: &str,
        _unit: &str,
        _owner: &str,
        _place: &str,
        _labels: &[String],
    ) -> Result<Resource> {
        not_possible("resource.edit")
    }
    /// `resource.edit`: refused if `expected` is not the current quantity, or if `quantity` would be
    /// less than what is held.
    async fn set_quantity(
        &self,
        _id: ObjectId,
        _expected: u32,
        _quantity: u32,
    ) -> Result<Resource> {
        not_possible("resource.edit")
    }
    /// `resource.delete`: refused while a confirmed reservation exists.
    async fn delete(&self, _id: ObjectId) -> Result<()> {
        not_possible("resource.delete")
    }
}
