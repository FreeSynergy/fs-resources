//! Offers to cover a need.
//!
//! Pattern: Strategy (see `resource.rs`). An offer has no read of its own in the core: they are read
//! per need, so a list without a need walks all needs.

use fs_admin_frontend::{BackendError, Outcome, Record};
use fs_resources_core::Offer;

use super::errors::CoreResult;
use super::input::{Input, bad};
use super::need;
use super::out::{Vals, done, made};
use super::ports::Ports;

fn record(p: &Ports, o: &Offer) -> Record {
    Vals::new()
        .put("need", p.iri(o.need))
        .put("by", &o.by)
        .num("quantity", o.quantity)
        .opt("resource", o.resource.map(|r| p.iri(r)))
        .put("state", o.state.as_str())
        .put("note", &o.note)
        .opt("reservation", o.reservation.map(|r| p.iri(r)))
        .record(p.iri(o.id))
}

/// `find`.
pub async fn list(p: &Ports, q: &Input<'_>) -> Result<Vec<Record>, BackendError> {
    let mut found = Vec::new();
    if let Some(n) = q.id("need")? {
        found.extend(p.needs.offers(n).await.shown()?);
    } else {
        for n in need::all(p, "", false).await? {
            found.extend(p.needs.offers(n.id).await.shown()?);
        }
    }
    Ok(found.iter().map(|o| record(p, o)).collect())
}

/// Carry out one command.
pub async fn run(p: &Ports, name: &str, i: &Input<'_>) -> Result<Outcome, BackendError> {
    match name {
        "create" => {
            let o = p
                .needs
                .offer(
                    i.need_id("need")?,
                    i.text("by").ok_or_else(|| bad("by"))?,
                    i.need_number("quantity")?,
                    i.id("resource")?,
                    i.text_or_empty("note"),
                )
                .await
                .shown()?;
            Ok(made(p.iri(o.id)))
        }
        "withdraw" => {
            p.needs.withdraw(i.need_id("id")?).await.shown()?;
            Ok(done())
        }
        "accept" | "decline" => {
            p.needs
                .decide(i.need_id("id")?, name == "accept")
                .await
                .shown()?;
            Ok(done())
        }
        _ => Err(BackendError::NotFound),
    }
}
