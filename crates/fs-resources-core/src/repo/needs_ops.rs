//! Needs: create, edit, close, delete, list, and keeping `covered` right.
//!
//! Pattern: State (use) — `covered` is always recomputed from the accepted offers (never counted up
//! and down), and `settle` turns it into the state: `covered` when it reaches the quantity, `open`
//! again when it falls below.

use fs_store::{ObjectId, Query, Timestamp};

use super::handle::ResourceRepo;
use super::reservation_ops::check_range;
use crate::error::{Error, Result};
use crate::model::{Cursor, Need, NeedState, Offer, OfferState, Page};
use crate::state::{close_need, settle};

fn check_when(from: Option<Timestamp>, to: Option<Timestamp>) -> Result<()> {
    match (from, to) {
        (None, None) => Ok(()),
        (Some(f), Some(t)) => check_range(f, t),
        _ => Err(Error::Invalid("a time needs both `from` and `to`")),
    }
}

fn check_need(what: &str, quantity: u32) -> Result<()> {
    if what.trim().is_empty() {
        return Err(Error::Invalid("a need says what is needed"));
    }
    if quantity == 0 {
        return Err(Error::Invalid("the quantity must not be zero"));
    }
    Ok(())
}

impl ResourceRepo {
    pub(super) async fn require_need(&self, id: ObjectId) -> Result<Need> {
        self.require("need", id).await
    }

    pub(super) async fn new_need(
        &self,
        what: &str,
        quantity: u32,
        unit: &str,
        when: (Option<Timestamp>, Option<Timestamp>),
        subject: &str,
    ) -> Result<Need> {
        check_need(what, quantity)?;
        check_when(when.0, when.1)?;
        let n = Need {
            id: ObjectId::new(),
            what: what.trim().into(),
            quantity,
            unit: unit.into(),
            from: when.0,
            to: when.1,
            subject: subject.into(),
            state: NeedState::Open,
            covered: 0,
        };
        self.save(&n).await?;
        Ok(n)
    }

    pub(super) async fn edit_need(
        &self,
        id: ObjectId,
        what: &str,
        quantity: u32,
        unit: &str,
        when: (Option<Timestamp>, Option<Timestamp>),
    ) -> Result<Need> {
        check_need(what, quantity)?;
        check_when(when.0, when.1)?;
        let mut n = self.require_need(id).await?;
        if n.state == NeedState::Closed {
            return Err(Error::InvalidTransition {
                what: "need",
                from: "closed",
                step: "edited",
            });
        }
        n.what = what.trim().into();
        n.quantity = quantity;
        n.unit = unit.into();
        (n.from, n.to) = when;
        n.state = settle(n.state, n.covered, n.quantity);
        self.save(&n).await?;
        Ok(n)
    }

    pub(super) async fn close_one(&self, id: ObjectId) -> Result<Need> {
        let mut n = self.require_need(id).await?;
        n.state = close_need(n.state)?;
        self.save(&n).await?;
        Ok(n)
    }

    pub(super) async fn offers_of(&self, need: ObjectId) -> Result<Vec<Offer>> {
        let mut all = self
            .find::<Offer>(Query::new().with_index("need", need.to_string()))
            .await?;
        all.sort_by_key(|o| o.id);
        Ok(all)
    }

    /// Recompute `covered` of a need from its accepted offers and settle its state.
    pub(super) async fn settle_need(&self, id: ObjectId) -> Result<Need> {
        let mut n = self.require_need(id).await?;
        let covered = self
            .offers_of(id)
            .await?
            .iter()
            .filter(|o| o.state == OfferState::Accepted)
            .map(|o| o.quantity)
            .sum();
        let state = settle(n.state, covered, n.quantity);
        if (covered, state) != (n.covered, n.state) {
            (n.covered, n.state) = (covered, state);
            self.save(&n).await?;
        }
        Ok(n)
    }

    pub(super) async fn delete_need(&self, id: ObjectId) -> Result<()> {
        self.require_need(id).await?;
        let offers = self.offers_of(id).await?;
        if offers.iter().any(|o| o.state == OfferState::Accepted) {
            return Err(Error::Conflict("an offer is accepted"));
        }
        for o in offers {
            self.erase::<Offer>(o.id).await?;
        }
        self.erase::<Need>(id).await
    }

    pub(super) async fn needs_page(
        &self,
        subject: &str,
        open_only: bool,
        cursor: Option<&Cursor>,
    ) -> Result<Page<Need>> {
        let mut q = Query::new();
        if !subject.is_empty() {
            q = q.with_index("subject", subject);
        }
        if open_only {
            q = q.with_index("state", "open");
        }
        let mut all = self.find::<Need>(q).await?;
        all.sort_by_key(|n| n.id);
        Ok(Page::cut(all, cursor, self.page_size))
    }
}
