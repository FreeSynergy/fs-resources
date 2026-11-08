//! `ResourceRepo` as a provider of [`Reservations`].
//!
//! Pattern: Adapter — each trait method takes the write lock where it reads before it writes, then
//! calls the operation of the same concern.

use async_trait::async_trait;
use fs_objects::{ObjectId, Query, Timestamp};

use super::handle::ResourceRepo;
use crate::error::Result;
use crate::model::{Cursor, FreePeriod, Page, Reservation};
use crate::state::Step;
use crate::traits::Reservations;

#[async_trait]
impl Reservations for ResourceRepo {
    async fn get(&self, id: ObjectId) -> Result<Option<Reservation>> {
        self.load(id).await
    }

    async fn of_resource(
        &self,
        resource: ObjectId,
        from: Timestamp,
        to: Timestamp,
        cursor: Option<&Cursor>,
    ) -> Result<Page<Reservation>> {
        let q = Query::new().with_index("resource", resource.to_string());
        let material = self
            .load::<crate::model::Resource>(resource)
            .await?
            .is_some_and(|r| r.kind == crate::model::ResourceKind::Material);
        self.page_of(q, |r| Self::touches(r, material, from, to), cursor)
            .await
    }

    async fn of_holder(&self, holder: &str, cursor: Option<&Cursor>) -> Result<Page<Reservation>> {
        let q = Query::new().with_index("holder", holder);
        self.page_of(q, |_| true, cursor).await
    }

    async fn free(
        &self,
        resource: ObjectId,
        from: Timestamp,
        to: Timestamp,
    ) -> Result<Vec<FreePeriod>> {
        self.free_of(resource, from, to).await
    }

    async fn request(
        &self,
        resource: ObjectId,
        quantity: u32,
        from: Timestamp,
        to: Timestamp,
        holder: &str,
        subject: &str,
        note: &str,
    ) -> Result<Reservation> {
        self.new_request(resource, quantity, from, to, holder, subject, note)
            .await
    }

    async fn confirm(&self, id: ObjectId) -> Result<Reservation> {
        let _write = self.lock.lock().await;
        self.confirm_request(id).await
    }

    async fn decline(&self, id: ObjectId) -> Result<Reservation> {
        let _write = self.lock.lock().await;
        self.end_request(id, Step::Decline).await
    }

    async fn cancel(&self, id: ObjectId) -> Result<Reservation> {
        let _write = self.lock.lock().await;
        self.end_request(id, Step::Cancel).await
    }

    async fn finish(&self, id: ObjectId, quantity: u32) -> Result<Reservation> {
        let _write = self.lock.lock().await;
        self.finish_reservation(id, quantity).await
    }

    async fn change(
        &self,
        id: ObjectId,
        quantity: u32,
        from: Timestamp,
        to: Timestamp,
    ) -> Result<Reservation> {
        let _write = self.lock.lock().await;
        self.change_reservation(id, quantity, from, to).await
    }
}
