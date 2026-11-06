//! Items: create, edit, set the quantity, get, list, delete.
//!
//! Pattern: Repository — `ResourceRepo` provides [`Items`]. `set_quantity` and `delete` read before
//! they write, so they run under the write lock and look at what is held through the sweep.

use async_trait::async_trait;
use fs_interfaces::Capabilities;
use fs_store::{ObjectId, Order, Query};

use super::handle::ResourceRepo;
use crate::error::{Error, Result};
use crate::model::{
    Cursor, Page, Reservation, ReservationState, Resource, ResourceFilter, ResourceKind,
};
use crate::sweep::peak;
use crate::traits::Items;

#[async_trait]
impl Items for ResourceRepo {
    fn capabilities(&self) -> Capabilities {
        self.support()
    }

    async fn get(&self, id: ObjectId) -> Result<Option<Resource>> {
        match self.load::<Resource>(id).await? {
            Some(r) => Ok(Some(self.shown(r).await?)),
            None => Ok(None),
        }
    }

    async fn list(
        &self,
        filter: &ResourceFilter,
        cursor: Option<&Cursor>,
    ) -> Result<Page<Resource>> {
        let offset = cursor.map_or(0, Cursor::offset);
        let mut q = Query::new().ordered(Order::NameAsc);
        for (key, value) in [
            ("owner", &filter.owner),
            ("place", &filter.place),
            ("label", &filter.label),
        ] {
            if !value.is_empty() {
                q = q.with_index(key, value.clone());
            }
        }
        if let Some(kind) = filter.kind {
            q = q.with_index("kind", kind.as_str());
        }
        if !filter.name.is_empty() {
            q = q.name_contains(filter.name.clone());
        }
        q.offset = offset;
        q.limit = Some(self.page_size + 1);
        let found = self.find::<Resource>(q).await?;
        let more = found.len() > self.page_size;
        let mut items = Vec::new();
        for r in found.into_iter().take(self.page_size) {
            items.push(self.shown(r).await?);
        }
        Ok(Page {
            items,
            next: more.then(|| Cursor::at(offset + self.page_size)),
        })
    }

    async fn create(
        &self,
        kind: ResourceKind,
        name: &str,
        quantity: u32,
        unit: &str,
        owner: &str,
        place: &str,
    ) -> Result<Resource> {
        if name.trim().is_empty() {
            return Err(Error::Invalid("a resource needs a name"));
        }
        if quantity == 0 {
            return Err(Error::Invalid("the quantity must not be zero"));
        }
        let r = Resource {
            id: ObjectId::new(),
            name: name.trim().into(),
            kind,
            quantity,
            unit: unit.into(),
            owner: owner.into(),
            place: place.into(),
            labels: vec![],
        };
        self.save(&r).await?;
        Ok(r)
    }

    async fn edit(
        &self,
        id: ObjectId,
        name: &str,
        unit: &str,
        owner: &str,
        place: &str,
        labels: &[String],
    ) -> Result<Resource> {
        if name.trim().is_empty() {
            return Err(Error::Invalid("a resource needs a name"));
        }
        let _write = self.lock.lock().await;
        let mut r = self.base_resource(id).await?;
        r.name = name.trim().into();
        r.unit = unit.into();
        r.owner = owner.into();
        r.place = place.into();
        r.labels = labels.iter().filter(|l| !l.is_empty()).cloned().collect();
        self.save(&r).await?;
        self.shown(r).await
    }

    async fn set_quantity(&self, id: ObjectId, expected: u32, quantity: u32) -> Result<Resource> {
        if quantity == 0 {
            return Err(Error::Invalid("the quantity must not be zero"));
        }
        let _write = self.lock.lock().await;
        let mut r = self.base_resource(id).await?;
        let used = self.used(&r).await?;
        if r.quantity.saturating_sub(used) != expected {
            return Err(Error::Conflict("the quantity is not what was expected"));
        }
        if peak(&self.holds(&r, None).await?) > u64::from(quantity) {
            return Err(Error::Conflict("less than what is held"));
        }
        // the stored base is what makes the shown quantity the one asked for
        r.quantity = quantity.saturating_add(used);
        self.save(&r).await?;
        self.shown(r).await
    }

    async fn delete(&self, id: ObjectId) -> Result<()> {
        let _write = self.lock.lock().await;
        self.require_resource(id).await?;
        let own = self
            .find::<Reservation>(Query::new().with_index("resource", id.to_string()))
            .await?;
        if own.iter().any(|r| r.state == ReservationState::Confirmed) {
            return Err(Error::Conflict("a confirmed reservation exists"));
        }
        for r in own {
            self.erase::<Reservation>(r.id).await?;
        }
        self.erase::<Resource>(id).await
    }
}
