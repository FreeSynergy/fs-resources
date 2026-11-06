//! The [`Reservations`] contract: asking, deciding, handing back, free periods.
//!
//! Pattern: Repository (interface) + Template method + Null Object — as [`Items`](super::Items):
//! reads are required, writes default to an explicit `Unsupported`.

use async_trait::async_trait;
use fs_store::{ObjectId, Timestamp};

use crate::error::Result;
use crate::model::{Cursor, FreePeriod, Page, Reservation};
use crate::unsupported::not_possible;

/// Reservations of resources.
#[async_trait]
pub trait Reservations: Send + Sync {
    /// `reservation.read`: one reservation, if it exists.
    async fn get(&self, id: ObjectId) -> Result<Option<Reservation>>;
    /// `reservation.read`: reservations of a resource that touch `[from, to)`, by start.
    async fn of_resource(
        &self,
        resource: ObjectId,
        from: Timestamp,
        to: Timestamp,
        cursor: Option<&Cursor>,
    ) -> Result<Page<Reservation>>;
    /// `reservation.read`: reservations of a person or group, by start.
    async fn of_holder(&self, holder: &str, cursor: Option<&Cursor>) -> Result<Page<Reservation>>;
    /// `reservation.read`: stretches of equal free quantity over `[from, to)`.
    async fn free(
        &self,
        resource: ObjectId,
        from: Timestamp,
        to: Timestamp,
    ) -> Result<Vec<FreePeriod>>;

    /// `reservation.request`: ask for a quantity; it holds nothing until confirmed.
    async fn request(
        &self,
        _resource: ObjectId,
        _quantity: u32,
        _from: Timestamp,
        _to: Timestamp,
        _holder: &str,
        _subject: &str,
        _note: &str,
    ) -> Result<Reservation> {
        not_possible("reservation.request")
    }
    /// `reservation.decide`: hold the quantity; refused if not enough is free.
    async fn confirm(&self, _id: ObjectId) -> Result<Reservation> {
        not_possible("reservation.decide")
    }
    /// `reservation.decide`: refuse a request.
    async fn decline(&self, _id: ObjectId) -> Result<Reservation> {
        not_possible("reservation.decide")
    }
    /// `reservation.request`: withdraw a request or a confirmed reservation.
    async fn cancel(&self, _id: ObjectId) -> Result<Reservation> {
        not_possible("reservation.request")
    }
    /// `reservation.decide`: thing, room: `quantity` came back, and it ends when all have.
    /// Material: `quantity` was used, the rest stays in stock, and it ends now.
    /// Refused once it has ended.
    async fn finish(&self, _id: ObjectId, _quantity: u32) -> Result<Reservation> {
        not_possible("reservation.decide")
    }
    /// `reservation.request`: one step; a confirmed reservation keeps what it holds, or nothing
    /// changes.
    async fn change(
        &self,
        _id: ObjectId,
        _quantity: u32,
        _from: Timestamp,
        _to: Timestamp,
    ) -> Result<Reservation> {
        not_possible("reservation.request")
    }
}
