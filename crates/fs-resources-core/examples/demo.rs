//! Resources in one go: lend a tent, use up paint, ask for hours, cover a need.
//!
//! Run with `cargo run -p fs-resources-core --example demo`.

use std::sync::Arc;

use fs_objects::{ObjectId, Timestamp, TursoStore};
use fs_resources_core::{Capacity, Error, ResourceKind, ResourceRepo};

fn hours(h: i64) -> Timestamp {
    Timestamp::from_timestamp(1_772_409_600 + h * 3600, 0).expect("a valid time")
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let store = Arc::new(TursoStore::open_in_memory().await?);
    // capacity and allocations live in a store of their own
    let private = Arc::new(TursoStore::open_in_memory().await?);
    let me = "https://example.org/people/ada";
    let repo = ResourceRepo::new(store.clone(), private.clone(), me).with_instance("example.org");

    // Things are lent and come back; two reservations may not hold more than exists.
    let tent = repo
        .items()
        .create(ResourceKind::Thing, "Tent", 2, "piece", me, "")
        .await?;
    let weekend = |n| {
        repo.reservations()
            .request(tent.id, n, hours(120), hours(168), me, "", "")
    };
    let first = repo.reservations().confirm(weekend(2).await?.id).await?;
    match repo.reservations().confirm(weekend(1).await?.id).await {
        Err(e @ Error::NotFree { .. }) => println!("second tent: {e}"),
        other => println!("unexpected: {other:?}"),
    }
    repo.reservations().finish(first.id, 2).await?;
    println!(
        "free again: {:?}",
        repo.reservations()
            .free(tent.id, hours(120), hours(168))
            .await?
    );

    // Material is used up: the stock goes down for good.
    let paint = repo
        .items()
        .create(ResourceKind::Material, "Paint", 10, "litre", me, "")
        .await?;
    let r = repo
        .reservations()
        .request(paint.id, 10, hours(0), hours(24), me, "", "")
        .await?;
    repo.reservations().confirm(r.id).await?;
    repo.reservations().finish(r.id, 4).await?;
    println!(
        "paint left: {}",
        repo.items().get(paint.id).await?.map_or(0, |p| p.quantity)
    );

    // My time is mine: set it, let someone ask, answer.
    let week = Capacity {
        id: ObjectId::new(),
        person: String::new(),
        from: hours(0),
        to: hours(24 * 28),
        minutes_per_week: 1200,
        note: String::new(),
    };
    repo.capacity().set(week).await?;
    let boss = ResourceRepo::new(store, private, "https://example.org/people/bo");
    let ask = boss
        .capacity()
        .propose(me, "https://example.org/tasks/1", hours(0), hours(168), 300)
        .await?;
    repo.capacity().answer(ask.id, true).await?;
    println!(
        "my load: {:?}",
        repo.capacity().load(hours(0), hours(168)).await?
    );

    // A need and an offer of our tent make a reservation.
    let need = repo
        .needs()
        .create(
            "a tent",
            1,
            "piece",
            Some(hours(200)),
            Some(hours(230)),
            "https://example.org/projects/fair",
        )
        .await?;
    let offer = repo
        .needs()
        .offer(need.id, me, 1, Some(tent.id), "")
        .await?;
    let offer = repo.needs().decide(offer.id, true).await?;
    println!("need covered, reservation {:?}", offer.reservation);
    Ok(())
}
