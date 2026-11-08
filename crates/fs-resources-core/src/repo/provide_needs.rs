//! `ResourceRepo` as a provider of [`Needs`].
//!
//! Pattern: Adapter — writes that read first take the write lock, then call the operation of the
//! same concern.

use async_trait::async_trait;
use fs_objects::{ObjectId, Timestamp};

use super::handle::ResourceRepo;
use crate::error::Result;
use crate::model::{Cursor, Need, Offer, Page};
use crate::state::OfferStep;
use crate::traits::Needs;

#[async_trait]
impl Needs for ResourceRepo {
    async fn get(&self, id: ObjectId) -> Result<Option<Need>> {
        self.load(id).await
    }

    async fn list(
        &self,
        subject: &str,
        open_only: bool,
        cursor: Option<&Cursor>,
    ) -> Result<Page<Need>> {
        self.needs_page(subject, open_only, cursor).await
    }

    async fn offers(&self, need: ObjectId) -> Result<Vec<Offer>> {
        self.offers_of(need).await
    }

    async fn create(
        &self,
        what: &str,
        quantity: u32,
        unit: &str,
        from: Option<Timestamp>,
        to: Option<Timestamp>,
        subject: &str,
    ) -> Result<Need> {
        self.new_need(what, quantity, unit, (from, to), subject)
            .await
    }

    async fn edit(
        &self,
        id: ObjectId,
        what: &str,
        quantity: u32,
        unit: &str,
        from: Option<Timestamp>,
        to: Option<Timestamp>,
    ) -> Result<Need> {
        let _write = self.lock.lock().await;
        self.edit_need(id, what, quantity, unit, (from, to)).await
    }

    async fn close(&self, id: ObjectId) -> Result<Need> {
        let _write = self.lock.lock().await;
        self.close_one(id).await
    }

    async fn delete(&self, id: ObjectId) -> Result<()> {
        let _write = self.lock.lock().await;
        self.delete_need(id).await
    }

    async fn offer(
        &self,
        need: ObjectId,
        by: &str,
        quantity: u32,
        resource: Option<ObjectId>,
        note: &str,
    ) -> Result<Offer> {
        let _write = self.lock.lock().await;
        self.new_offer(need, by, quantity, resource, note).await
    }

    async fn withdraw(&self, offer: ObjectId) -> Result<Offer> {
        let _write = self.lock.lock().await;
        self.end_offer(offer, OfferStep::Withdraw).await
    }

    async fn decide(&self, offer: ObjectId, accept: bool) -> Result<Offer> {
        let _write = self.lock.lock().await;
        if accept {
            self.accept_offer(offer).await
        } else {
            self.end_offer(offer, OfferStep::Decline).await
        }
    }
}
