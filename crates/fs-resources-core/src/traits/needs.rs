//! The [`Needs`] contract: what is needed, and who offers it.
//!
//! Pattern: Repository (interface) + Template method + Null Object — as [`Items`](super::Items).

use async_trait::async_trait;
use fs_objects::{ObjectId, Timestamp};

use crate::error::Result;
use crate::model::{Cursor, Need, Offer, Page};
use crate::unsupported::not_possible;

/// Needs and the offers made for them.
#[async_trait]
pub trait Needs: Send + Sync {
    /// `need.read`: one need, if it exists.
    async fn get(&self, id: ObjectId) -> Result<Option<Need>>;
    /// `need.read`: needs of `subject` (empty = all), by creation; `open_only` keeps the open ones.
    async fn list(
        &self,
        subject: &str,
        open_only: bool,
        cursor: Option<&Cursor>,
    ) -> Result<Page<Need>>;
    /// `need.read`: the offers made for a need, by creation.
    async fn offers(&self, need: ObjectId) -> Result<Vec<Offer>>;

    /// `need.create`: a new need; with a time, both `from` and `to` are set and `to > from`.
    async fn create(
        &self,
        _what: &str,
        _quantity: u32,
        _unit: &str,
        _from: Option<Timestamp>,
        _to: Option<Timestamp>,
        _subject: &str,
    ) -> Result<Need> {
        not_possible("need.create")
    }
    /// `need.create`: replace what, quantity, unit and time.
    async fn edit(
        &self,
        _id: ObjectId,
        _what: &str,
        _quantity: u32,
        _unit: &str,
        _from: Option<Timestamp>,
        _to: Option<Timestamp>,
    ) -> Result<Need> {
        not_possible("need.create")
    }
    /// `need.create`: no more offers.
    async fn close(&self, _id: ObjectId) -> Result<Need> {
        not_possible("need.create")
    }
    /// `need.create`: delete a need and its offers; refused while an offer is accepted.
    async fn delete(&self, _id: ObjectId) -> Result<()> {
        not_possible("need.create")
    }
    /// `offer.make`: offer to cover (part of) a need; refused on a closed need.
    async fn offer(
        &self,
        _need: ObjectId,
        _by: &str,
        _quantity: u32,
        _resource: Option<ObjectId>,
        _note: &str,
    ) -> Result<Offer> {
        not_possible("offer.make")
    }
    /// `offer.make`: take an offer back; an accepted one gives up its reservation.
    async fn withdraw(&self, _offer: ObjectId) -> Result<Offer> {
        not_possible("offer.make")
    }
    /// `offer.decide`: accept or decline. Accepting makes a confirmed reservation if the offer names
    /// a resource and the need has a time; refused, with nothing changed, if that is not free.
    async fn decide(&self, _offer: ObjectId, _accept: bool) -> Result<Offer> {
        not_possible("offer.decide")
    }
}
