//! Reservations: request, decide, cancel, read, free periods.
//!
//! Pattern: State (use) — every change of state goes through `reservation_next`; confirming checks
//! the sweep. The caller holds the write lock (see `provide_reservations`).

use fs_store::{ObjectId, Query, Timestamp};

use super::handle::ResourceRepo;
use crate::error::{Error, Result};
use crate::model::Offer;
use crate::model::{Cursor, FreePeriod, Page, Reservation, ReservationState};
use crate::state::{Step, reservation_next};
use crate::sweep::{free_periods, overlaps};

pub(super) fn check_range(from: Timestamp, to: Timestamp) -> Result<()> {
    if to <= from {
        return Err(Error::Invalid("`to` must be after `from`"));
    }
    Ok(())
}

impl ResourceRepo {
    #[allow(clippy::too_many_arguments)]
    pub(super) async fn new_request(
        &self,
        resource: ObjectId,
        quantity: u32,
        from: Timestamp,
        to: Timestamp,
        holder: &str,
        subject: &str,
        note: &str,
    ) -> Result<Reservation> {
        if quantity == 0 {
            return Err(Error::Invalid("the quantity must not be zero"));
        }
        check_range(from, to)?;
        if holder.is_empty() {
            return Err(Error::Invalid("a reservation needs a holder"));
        }
        self.require_resource(resource).await?;
        let r = Reservation {
            id: ObjectId::new(),
            resource,
            quantity,
            from,
            to,
            holder: holder.into(),
            state: ReservationState::Requested,
            subject: subject.into(),
            note: note.into(),
            done: 0,
        };
        self.save(&r).await?;
        Ok(r)
    }

    pub(super) async fn confirm_request(&self, id: ObjectId) -> Result<Reservation> {
        let mut r = self.require_reservation(id).await?;
        r.state = reservation_next(r.state, Step::Confirm)?;
        let resource = self.require_resource(r.resource).await?;
        self.ensure_fits(&resource, &r).await?;
        self.save(&r).await?;
        Ok(r)
    }

    /// Decline or cancel: no hold is taken, so nothing needs to fit.
    pub(super) async fn end_request(&self, id: ObjectId, step: Step) -> Result<Reservation> {
        let mut r = self.require_reservation(id).await?;
        r.state = reservation_next(r.state, step)?;
        if step == Step::Cancel && r.done > 0 {
            return Err(Error::Conflict("part of it has already come back"));
        }
        if step == Step::Cancel && self.held_by_offer(id).await? {
            return Err(Error::Conflict(
                "an accepted offer holds it; withdraw the offer",
            ));
        }
        self.save(&r).await?;
        Ok(r)
    }

    async fn held_by_offer(&self, reservation: ObjectId) -> Result<bool> {
        let q = Query::new()
            .with_index("reservation", reservation.to_string())
            .with_index("state", "accepted");
        Ok(!self.find::<Offer>(q).await?.is_empty())
    }

    pub(super) async fn page_of(
        &self,
        q: Query,
        keep: impl Fn(&Reservation) -> bool,
        cursor: Option<&Cursor>,
    ) -> Result<Page<Reservation>> {
        let mut all: Vec<_> = self
            .find::<Reservation>(q)
            .await?
            .into_iter()
            .filter(|r| keep(r))
            .collect();
        all.sort_by_key(|r| (r.from, r.id));
        Ok(Page::cut(all, cursor, self.page_size))
    }

    pub(super) async fn free_of(
        &self,
        resource: ObjectId,
        from: Timestamp,
        to: Timestamp,
    ) -> Result<Vec<FreePeriod>> {
        check_range(from, to)?;
        let res = self.require_resource(resource).await?;
        let holds = self.holds(&res, None).await?;
        Ok(free_periods(res.quantity, &holds, from, to))
    }

    /// Whether `r` is part of `[from, to)`: a held reservation of material has no end, so it counts
    /// from its start on, as `free` counts it.
    pub(super) fn touches(r: &Reservation, material: bool, from: Timestamp, to: Timestamp) -> bool {
        if material && r.state == ReservationState::Confirmed {
            return r.from < to;
        }
        overlaps(r.from, r.to, from, to)
    }
}
