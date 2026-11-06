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
            Some(r) if !r.deleting => Ok(Some(self.shown(r).await?)),
            _ => Ok(None),
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
        // Hidden resources (being deleted) are skipped while scanning raw positions, so no row is
        // repeated and a page is only short when the list really ends. The cursor is a raw position.
        let (mut pos, mut items, mut next) = (offset, Vec::new(), None);
        'scan: loop {
            q.offset = pos.min(1 << 40); // the store counts in i64
            q.limit = Some(self.page_size + 1);
            let chunk = self.find::<Resource>(q.clone()).await?;
            let n = chunk.len();
            for (i, r) in chunk.into_iter().enumerate().filter(|(_, r)| !r.deleting) {
                if items.len() == self.page_size {
                    next = Some(Cursor::at(pos.saturating_add(i)));
                    break 'scan;
                }
                items.push(self.shown(r).await?);
            }
            if n <= self.page_size {
                break;
            }
            pos = pos.saturating_add(n);
        }
        Ok(Page { items, next })
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
            deleting: false,
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
        r.quantity = quantity
            .checked_add(used)
            .ok_or(Error::Invalid("the quantity is too large"))?;
        self.save(&r).await?;
        self.shown(r).await
    }

    async fn delete(&self, id: ObjectId) -> Result<()> {
        let _write = self.lock.lock().await;
        let mut resource = self.load::<Resource>(id).await?.ok_or(Error::NotFound {
            what: "resource",
            id,
        })?;
        let own = self
            .find::<Reservation>(Query::new().with_index("resource", id.to_string()))
            .await?;
        if !resource.deleting {
            if own.iter().any(|r| r.state == ReservationState::Confirmed) {
                return Err(Error::Conflict("a confirmed reservation exists"));
            }
            // Mark first: from here the resource is hidden, its stock frozen and no reservation
            // is taken, so a fault below leaves nothing visible, and deleting again resumes.
            resource.deleting = true;
            self.save(&resource).await?;
        }
        for r in own {
            self.erase::<Reservation>(r.id).await?;
        }
        self.erase::<Resource>(id).await
    }
}
