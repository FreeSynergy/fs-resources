//! Resources: things, rooms, material.
//!
//! Pattern: Strategy (one module per data type, same two functions `list` and `run`) — the command
//! names are those of the table in the core (`describe::COMMANDS`).

use fs_admin_frontend::{BackendError, Outcome, Record};
use fs_resources_core::{Cursor, Resource, ResourceFilter, ResourceKind};

use super::errors::CoreResult;
use super::input::{Input, bad, stamp};
use super::out::{Vals, done, made};
use super::ports::Ports;

fn kind_of(key: &str) -> Option<ResourceKind> {
    match key {
        "thing" => Some(ResourceKind::Thing),
        "room" => Some(ResourceKind::Room),
        "material" => Some(ResourceKind::Material),
        _ => None,
    }
}

fn record(p: &Ports, r: &Resource) -> Record {
    Vals::new()
        .put("name", &r.name)
        .put("kind", r.kind.as_str())
        .num("quantity", r.quantity)
        .put("unit", &r.unit)
        .put("owner", &r.owner)
        .put("place", &r.place)
        .put("labels", r.labels.join(", "))
        .record(p.iri(r.id))
}

/// `find`: every page, in the core's order.
pub async fn list(p: &Ports, q: &Input<'_>) -> Result<Vec<Record>, BackendError> {
    let filter = ResourceFilter {
        kind: match q.text("kind") {
            Some(k) => Some(kind_of(k).ok_or_else(|| bad("kind"))?),
            None => None,
        },
        owner: q.text_or_empty("owner").to_owned(),
        place: q.text_or_empty("place").to_owned(),
        label: q.text_or_empty("label").to_owned(),
        name: q.text_or_empty("term").to_owned(),
    };
    let mut out = Vec::new();
    let mut cursor: Option<Cursor> = None;
    loop {
        let page = p.items.list(&filter, cursor.as_ref()).await.shown()?;
        out.extend(page.items.iter().map(|r| record(p, r)));
        match page.next {
            Some(next) => cursor = Some(next),
            None => return Ok(out),
        }
    }
}

/// Carry out one command.
pub async fn run(p: &Ports, name: &str, i: &Input<'_>) -> Result<Outcome, BackendError> {
    match name {
        "create" => {
            let kind = i
                .text("kind")
                .and_then(kind_of)
                .ok_or_else(|| bad("kind"))?;
            let made_one = p
                .items
                .create(
                    kind,
                    i.text("name").ok_or_else(|| bad("name"))?,
                    i.need_number("quantity")?,
                    i.text_or_empty("unit"),
                    i.text_or_empty("owner"),
                    i.text_or_empty("place"),
                )
                .await
                .shown()?;
            Ok(made(p.iri(made_one.id)))
        }
        "change" => {
            i.only(&["name", "unit", "owner", "place", "labels", "quantity"])?;
            let id = i.need_id("id")?;
            let now = p
                .items
                .get(id)
                .await
                .shown()?
                .ok_or(BackendError::NotFound)?;
            // missing = unchanged, empty = cleared (a name cannot be cleared: the core refuses it)
            let pick = |field: &str, old: &str| {
                if i.has(field) {
                    i.text_or_empty(field).to_owned()
                } else {
                    old.to_owned()
                }
            };
            let labels = if i.has("labels") {
                i.words("labels")
            } else {
                now.labels.clone()
            };
            p.items
                .edit(
                    id,
                    &pick("name", &now.name),
                    &pick("unit", &now.unit),
                    &pick("owner", &now.owner),
                    &pick("place", &now.place),
                    &labels,
                )
                .await
                .shown()?;
            if let Some(q) = i.number("quantity")? {
                p.items.set_quantity(id, now.quantity, q).await.shown()?;
            } else if i.has("quantity") {
                return Err(bad("quantity"));
            }
            Ok(made(p.iri(id)))
        }
        "delete" => {
            p.items.delete(i.need_id("id")?).await.shown()?;
            Ok(done())
        }
        "free" => {
            let periods = p
                .reservations
                .free(i.need_id("id")?, i.need_time("from")?, i.need_time("to")?)
                .await
                .shown()?;
            let mut v = Vals::new();
            for f in periods {
                v = v
                    .push("from", stamp(f.from))
                    .push("to", stamp(f.to))
                    .num("free", f.free);
            }
            Ok(v.outcome())
        }
        _ => Err(BackendError::NotFound),
    }
}
