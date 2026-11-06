//! Offers: make, withdraw, decide.
//!
//! Pattern: State (use) — every change goes through `offer_next`; repeating `decide` or `withdraw`
//! to the state the offer is already in only finishes the follow-up work. Accepting an offer that names a
//! resource, on a need with a time, books a confirmed reservation through the same fit check as
//! `confirm`. The order is reservation, offer, need: a crash leaves at worst a hold that no offer
//! refers to (it can be cancelled), never an accepted offer without its hold.

use fs_store::ObjectId;

use super::handle::ResourceRepo;
use crate::error::{Error, Result};
use crate::model::{Need, NeedState, Offer, OfferState, Reservation, ReservationState};
use crate::state::{OfferStep, Step, offer_next, reservation_next};

impl ResourceRepo {
    async fn require_offer(&self, id: ObjectId) -> Result<Offer> {
        self.require("offer", id).await
    }

    pub(super) async fn new_offer(
        &self,
        need: ObjectId,
        by: &str,
        quantity: u32,
        resource: Option<ObjectId>,
        note: &str,
    ) -> Result<Offer> {
        if quantity == 0 {
            return Err(Error::Invalid("the quantity must not be zero"));
        }
        if by.is_empty() {
            return Err(Error::Invalid("an offer needs someone who offers"));
        }
        if self.require_need(need).await?.state == NeedState::Closed {
            return Err(Error::Conflict("the need is closed"));
        }
        if let Some(r) = resource {
            self.require_resource(r).await?;
        }
        let o = Offer {
            id: ObjectId::new(),
            need,
            by: by.into(),
            quantity,
            resource,
            state: OfferState::Offered,
            note: note.into(),
            reservation: None,
        };
        self.save(&o).await?;
        Ok(o)
    }

    /// Withdraw or decline: an accepted offer gives up its reservation and the need is settled.
    pub(super) async fn end_offer(&self, id: ObjectId, step: OfferStep) -> Result<Offer> {
        let mut o = self.require_offer(id).await?;
        let target = match step {
            OfferStep::Withdraw => OfferState::Withdrawn,
            _ => OfferState::Declined,
        };
        // Repeating the step finishes what a crash left behind: the offer is saved first, so the
        // release and the settling below are the parts that can be missing.
        if o.state != target {
            o.state = offer_next(o.state, step)?;
            self.save(&o).await?;
        }
        self.release(&o).await?;
        self.settle_need(o.need).await?;
        Ok(o)
    }

    async fn release(&self, offer: &Offer) -> Result<()> {
        let Some(id) = offer.reservation else {
            return Ok(());
        };
        let Some(mut r) = self.load::<Reservation>(id).await? else {
            return Ok(());
        };
        if r.state == ReservationState::Confirmed && r.done == 0 {
            r.state = reservation_next(r.state, Step::Cancel)?;
            self.save(&r).await?;
        }
        Ok(())
    }

    pub(super) async fn accept_offer(&self, id: ObjectId) -> Result<Offer> {
        let mut o = self.require_offer(id).await?;
        if o.state == OfferState::Accepted {
            // a repeated accept settles what a crash left unfinished
            self.settle_need(o.need).await?;
            return Ok(o);
        }
        let need = self.require_need(o.need).await?;
        let accepted = offer_next(o.state, OfferStep::Accept)?;
        if need.state == NeedState::Closed {
            return Err(Error::Conflict("the need is closed"));
        }
        if let Some(r) = self.reservation_for(&need, &o) {
            let resource = self.require_resource(r.resource).await?;
            self.ensure_fits(&resource, &r).await?;
            // The id is written into the offer (still `offered`) before the reservation exists, so
            // a retry after a fault reuses it instead of booking a second one.
            o.reservation = Some(r.id);
            self.save(&o).await?;
            self.save(&r).await?;
        }
        o.state = accepted;
        self.save(&o).await?;
        self.settle_need(o.need).await?;
        Ok(o)
    }

    /// The confirmed reservation an accepted offer makes, if it names a resource and the need has
    /// a time. It keeps the id the offer already holds, if any. Nothing is written here.
    fn reservation_for(&self, need: &Need, offer: &Offer) -> Option<Reservation> {
        let (Some(resource), Some(from), Some(to)) = (offer.resource, need.from, need.to) else {
            return None;
        };
        let holder = if need.subject.is_empty() {
            self.iri(need.id)
        } else {
            need.subject.clone()
        };
        let r = Reservation {
            id: offer.reservation.unwrap_or_default(),
            resource,
            quantity: offer.quantity,
            from,
            to,
            holder,
            state: ReservationState::Confirmed,
            subject: String::new(),
            note: format!("offer {}", offer.id),
            done: 0,
        };
        Some(r)
    }
}
