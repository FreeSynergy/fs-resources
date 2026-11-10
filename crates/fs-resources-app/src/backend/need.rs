//! Needs.
//!
//! Pattern: Strategy (see `resource.rs`).

use fs_admin_frontend::{BackendError, Outcome, Record};
use fs_resources_core::{Cursor, Need};

use super::errors::CoreResult;
use super::input::{Input, bad, stamp};
use super::out::{Vals, done, made};
use super::ports::Ports;

pub(super) fn record(p: &Ports, n: &Need) -> Record {
    Vals::new()
        .put("what", &n.what)
        .num("quantity", n.quantity)
        .put("unit", &n.unit)
        .opt("from", n.from.map(stamp))
        .opt("to", n.to.map(stamp))
        .put("subject", &n.subject)
        .put("state", n.state.as_str())
        .num("covered", n.covered)
        .record(p.iri(n.id))
}

/// Every need of a subject (empty = all), open ones only if asked.
pub(super) async fn all(
    p: &Ports,
    subject: &str,
    open_only: bool,
) -> Result<Vec<Need>, BackendError> {
    let mut out = Vec::new();
    let mut cursor: Option<Cursor> = None;
    loop {
        let page = p
            .needs
            .list(subject, open_only, cursor.as_ref())
            .await
            .shown()?;
        out.extend(page.items);
        match page.next {
            Some(next) => cursor = Some(next),
            None => return Ok(out),
        }
    }
}

/// `find`.
pub async fn list(p: &Ports, q: &Input<'_>) -> Result<Vec<Record>, BackendError> {
    let needs = all(p, q.text_or_empty("subject"), q.is("scope", "open")).await?;
    Ok(needs.iter().map(|n| record(p, n)).collect())
}

/// Carry out one command.
pub async fn run(p: &Ports, name: &str, i: &Input<'_>) -> Result<Outcome, BackendError> {
    match name {
        "create" => {
            let n = p
                .needs
                .create(
                    i.text("what").ok_or_else(|| bad("what"))?,
                    i.need_number("quantity")?,
                    i.text_or_empty("unit"),
                    i.time("from")?,
                    i.time("to")?,
                    i.text_or_empty("subject"),
                )
                .await
                .shown()?;
            Ok(made(p.iri(n.id)))
        }
        "change" => {
            i.only(&["what", "quantity", "unit", "from", "to"])?;
            let id = i.need_id("id")?;
            let now = p
                .needs
                .get(id)
                .await
                .shown()?
                .ok_or(BackendError::NotFound)?;
            let what = if i.has("what") {
                i.text("what").ok_or_else(|| bad("what"))?
            } else {
                &now.what
            };
            let quantity = if i.has("quantity") {
                i.need_number("quantity")?
            } else {
                now.quantity
            };
            let unit = if i.has("unit") {
                i.text_or_empty("unit")
            } else {
                &now.unit
            };
            // missing = unchanged, empty = no time any more
            let from = if i.has("from") {
                i.time("from")?
            } else {
                now.from
            };
            let to = if i.has("to") { i.time("to")? } else { now.to };
            p.needs
                .edit(id, what, quantity, unit, from, to)
                .await
                .shown()?;
            Ok(made(p.iri(id)))
        }
        "close" => {
            p.needs.close(i.need_id("id")?).await.shown()?;
            Ok(done())
        }
        "delete" => {
            p.needs.delete(i.need_id("id")?).await.shown()?;
            Ok(done())
        }
        _ => Err(BackendError::NotFound),
    }
}
