//! A person's own capacity, load and sharing. Every call acts for the person the repository was built for.
//!
//! Pattern: Strategy (see `resource.rs`). Groups are another program's: a group is named by a
//! `urn:uuid:` IRI here (provisional, DESIGN.md) because this program cannot know the group's own IRI.

use fs_admin_frontend::{BackendError, Outcome, Record};
use fs_objects::ObjectId;
use fs_resources_core::Capacity;

use super::errors::CoreResult;
use super::input::{Input, bad, forever, stamp};
use super::out::{Vals, done, made};
use super::ports::Ports;

fn record(p: &Ports, c: &Capacity) -> Record {
    Vals::new()
        .put("person", &c.person)
        .put("from", stamp(c.from))
        .put("to", stamp(c.to))
        .num("minutesPerWeek", c.minutes_per_week)
        .put("note", &c.note)
        .record(p.iri(c.id))
}

fn group_iri(id: ObjectId) -> String {
    format!("urn:uuid:{id}")
}

/// `find`.
pub async fn list(p: &Ports, q: &Input<'_>) -> Result<Vec<Record>, BackendError> {
    let (start, end) = forever();
    let mine = p
        .capacities
        .mine(
            q.time("from")?.unwrap_or(start),
            q.time("to")?.unwrap_or(end),
        )
        .await
        .shown()?;
    Ok(mine.iter().map(|c| record(p, c)).collect())
}

/// Carry out one command.
pub async fn run(p: &Ports, name: &str, i: &Input<'_>) -> Result<Outcome, BackendError> {
    match name {
        "set" => {
            let saved = p
                .capacities
                .set(Capacity {
                    id: ObjectId::new(),
                    person: String::new(),
                    from: i.need_time("from")?,
                    to: i.need_time("to")?,
                    minutes_per_week: i.need_number("minutesPerWeek")?,
                    note: i.text_or_empty("note").to_owned(),
                })
                .await
                .shown()?;
            Ok(made(p.iri(saved.id)))
        }
        "change" => {
            i.only(&["from", "to", "minutesPerWeek", "note"])?;
            let id = i.need_id("id")?;
            let (start, end) = forever();
            let now = p
                .capacities
                .mine(start, end)
                .await
                .shown()?
                .into_iter()
                .find(|c| c.id == id)
                .ok_or(BackendError::NotFound)?;
            let next = Capacity {
                from: if i.has("from") {
                    i.need_time("from")?
                } else {
                    now.from
                },
                to: if i.has("to") {
                    i.need_time("to")?
                } else {
                    now.to
                },
                minutes_per_week: if i.has("minutesPerWeek") {
                    i.need_number("minutesPerWeek")?
                } else {
                    now.minutes_per_week
                },
                note: if i.has("note") {
                    i.text_or_empty("note").to_owned()
                } else {
                    now.note.clone()
                },
                ..now
            };
            p.capacities.set(next).await.shown()?;
            Ok(made(p.iri(id)))
        }
        "delete" => {
            p.capacities.remove(i.need_id("id")?).await.shown()?;
            Ok(done())
        }
        "load" => {
            let l = p
                .capacities
                .load(i.need_time("from")?, i.need_time("to")?)
                .await
                .shown()?;
            Ok(Vals::new()
                .num("capacityMinutes", l.capacity_minutes)
                .num("allocatedMinutes", l.allocated_minutes)
                .outcome())
        }
        "share" => {
            let on = match i.text("share") {
                Some("on") => true,
                Some("off") => false,
                _ => return Err(bad("share")),
            };
            p.capacities
                .share_with_group(i.need_id("group")?, on)
                .await
                .shown()?;
            Ok(done())
        }
        "sharedGroups" => {
            let mut v = Vals::new();
            for g in p.capacities.shared_groups().await.shown()? {
                v = v.push("groups", group_iri(g));
            }
            Ok(v.outcome())
        }
        "groupLoad" => {
            let l = p
                .capacities
                .group_load(
                    i.need_id("group")?,
                    i.need_time("from")?,
                    i.need_time("to")?,
                )
                .await
                .shown()?;
            Ok(Vals::new()
                .num("hidden", l.hidden)
                .num("allocationsHidden", l.allocations_hidden)
                .num("capacityMinutes", l.load.capacity_minutes)
                .num("allocatedMinutes", l.load.allocated_minutes)
                .outcome())
        }
        _ => Err(BackendError::NotFound),
    }
}
