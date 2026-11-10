//! Allocations of a person's hours.
//!
//! Pattern: Strategy (see `resource.rs`).

use fs_admin_frontend::{BackendError, Outcome, Record};
use fs_resources_core::Allocation;

use super::errors::CoreResult;
use super::input::{Input, bad, forever, stamp};
use super::out::{Vals, done, made};
use super::ports::Ports;

fn record(p: &Ports, a: &Allocation) -> Record {
    Vals::new()
        .put("person", &a.person)
        .put("subject", &a.subject)
        .put("from", stamp(a.from))
        .put("to", stamp(a.to))
        .num("minutes", a.minutes)
        .put("state", a.state.as_str())
        .put("proposer", &a.proposer)
        .record(p.iri(a.id))
}

/// `find`: asked of me, asked by me, or both (empty scope), each once.
pub async fn list(p: &Ports, q: &Input<'_>) -> Result<Vec<Record>, BackendError> {
    let (start, end) = forever();
    let (from, to) = (
        q.time("from")?.unwrap_or(start),
        q.time("to")?.unwrap_or(end),
    );
    let scope = q.text("scope");
    let mut found = Vec::new();
    if scope != Some("proposed") {
        found.extend(p.capacities.allocations(from, to).await.shown()?);
    }
    if scope != Some("mine") {
        for a in p.capacities.proposed(from, to).await.shown()? {
            if !found.iter().any(|f: &Allocation| f.id == a.id) {
                found.push(a);
            }
        }
    }
    found.sort_by_key(|a| (a.from, a.id));
    Ok(found.iter().map(|a| record(p, a)).collect())
}

/// Carry out one command.
pub async fn run(p: &Ports, name: &str, i: &Input<'_>) -> Result<Outcome, BackendError> {
    match name {
        "propose" => {
            let a = p
                .capacities
                .propose(
                    i.text("person").ok_or_else(|| bad("person"))?,
                    i.text("subject").ok_or_else(|| bad("subject"))?,
                    i.need_time("from")?,
                    i.need_time("to")?,
                    i.need_number("minutes")?,
                )
                .await
                .shown()?;
            Ok(made(p.iri(a.id)))
        }
        "accept" | "decline" => {
            p.capacities
                .answer(i.need_id("id")?, name == "accept")
                .await
                .shown()?;
            Ok(done())
        }
        "withdraw" => {
            p.capacities.withdraw(i.need_id("id")?).await.shown()?;
            Ok(done())
        }
        _ => Err(BackendError::NotFound),
    }
}
