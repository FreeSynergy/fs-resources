//! Reservations.
//!
//! Pattern: Strategy (see `resource.rs`). A list without a resource or holder walks the resources,
//! because the core has no "all reservations" read; the frontend's list is `find` with an empty term.

use fs_admin_frontend::{BackendError, Outcome, Record};
use fs_resources_core::{Cursor, Page, Reservation, ResourceFilter};

use super::errors::CoreResult;
use super::input::{Input, bad, forever, stamp};
use super::out::{Vals, done, made};
use super::ports::Ports;

fn record(p: &Ports, r: &Reservation) -> Record {
    Vals::new()
        .put("resource", p.iri(r.resource))
        .num("quantity", r.quantity)
        .put("from", stamp(r.from))
        .put("to", stamp(r.to))
        .put("holder", &r.holder)
        .put("state", r.state.as_str())
        .put("subject", &r.subject)
        .put("note", &r.note)
        .num("done", r.done)
        .record(p.iri(r.id))
}

/// `find`.
pub async fn list(p: &Ports, q: &Input<'_>) -> Result<Vec<Record>, BackendError> {
    let (start, end) = forever();
    let (from, to) = (
        q.time("from")?.unwrap_or(start),
        q.time("to")?.unwrap_or(end),
    );
    let mut found: Vec<Reservation> = Vec::new();
    if let Some(resource) = q.id("resource")? {
        found = pages(|c| async move {
            p.reservations
                .of_resource(resource, from, to, c.as_ref())
                .await
        })
        .await?;
    } else if let Some(holder) = q.text("holder") {
        found =
            pages(|c| async move { p.reservations.of_holder(holder, c.as_ref()).await }).await?;
    } else {
        let mut cursor: Option<Cursor> = None;
        loop {
            let page = p
                .items
                .list(&ResourceFilter::default(), cursor.as_ref())
                .await
                .shown()?;
            for r in &page.items {
                let id = r.id;
                found.extend(
                    pages(|c| async move { p.reservations.of_resource(id, from, to, c.as_ref()).await }).await?,
                );
            }
            match page.next {
                Some(next) => cursor = Some(next),
                None => break,
            }
        }
    }
    found.sort_by_key(|r| (r.from, r.id));
    Ok(found.iter().map(|r| record(p, r)).collect())
}

/// Every page of a read that takes a cursor; the closure gets the cursor by value so its future owns it.
async fn pages<F, Fut>(mut one: F) -> Result<Vec<Reservation>, BackendError>
where
    F: FnMut(Option<Cursor>) -> Fut,
    Fut: std::future::Future<Output = fs_resources_core::Result<Page<Reservation>>>,
{
    let mut out = Vec::new();
    let mut cursor: Option<Cursor> = None;
    loop {
        let page = one(cursor.take()).await.shown()?;
        out.extend(page.items);
        match page.next {
            Some(next) => cursor = Some(next),
            None => return Ok(out),
        }
    }
}

/// Carry out one command.
pub async fn run(p: &Ports, name: &str, i: &Input<'_>) -> Result<Outcome, BackendError> {
    match name {
        "request" => {
            let r = p
                .reservations
                .request(
                    i.need_id("resource")?,
                    i.need_number("quantity")?,
                    i.need_time("from")?,
                    i.need_time("to")?,
                    i.text("holder").ok_or_else(|| bad("holder"))?,
                    i.text_or_empty("subject"),
                    i.text_or_empty("note"),
                )
                .await
                .shown()?;
            Ok(made(p.iri(r.id)))
        }
        "change" => {
            i.only(&["quantity", "from", "to"])?;
            let id = i.need_id("id")?;
            let now = p
                .reservations
                .get(id)
                .await
                .shown()?
                .ok_or(BackendError::NotFound)?;
            let quantity = if i.has("quantity") {
                i.need_number("quantity")?
            } else {
                now.quantity
            };
            let from = if i.has("from") {
                i.need_time("from")?
            } else {
                now.from
            };
            let to = if i.has("to") {
                i.need_time("to")?
            } else {
                now.to
            };
            p.reservations
                .change(id, quantity, from, to)
                .await
                .shown()?;
            Ok(made(p.iri(id)))
        }
        "confirm" => {
            step(
                p,
                i,
                |p, id| async move { p.reservations.confirm(id).await },
            )
            .await
        }
        "decline" => {
            step(
                p,
                i,
                |p, id| async move { p.reservations.decline(id).await },
            )
            .await
        }
        "cancel" => step(p, i, |p, id| async move { p.reservations.cancel(id).await }).await,
        "finish" => {
            let quantity = i.need_number("quantity")?;
            step(p, i, move |p, id| async move {
                p.reservations.finish(id, quantity).await
            })
            .await
        }
        _ => Err(BackendError::NotFound),
    }
}

async fn step<F, Fut>(p: &Ports, i: &Input<'_>, f: F) -> Result<Outcome, BackendError>
where
    F: FnOnce(Ports, fs_objects::ObjectId) -> Fut,
    Fut: std::future::Future<Output = fs_resources_core::Result<Reservation>>,
{
    f(p.clone(), i.need_id("id")?).await.shown()?;
    Ok(done())
}
