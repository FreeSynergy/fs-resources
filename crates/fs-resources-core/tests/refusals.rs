//! Input the contract refuses: zero quantity, zero duration, `to <= from`, delete while confirmed.

mod common;

use common::{ME, ask, hold, make, person, setup, t};
use fs_objects::ObjectId;
use fs_resources_core::{Capacity, Error, ResourceKind};

#[allow(clippy::needless_pass_by_value)] // callers pass the result by value; a reference would touch every call site
fn invalid<T: std::fmt::Debug>(r: Result<T, Error>) {
    assert!(matches!(r, Err(Error::Invalid(_))), "{r:?}");
}

#[tokio::test]
async fn zero_quantity_is_refused_everywhere() {
    let (repo, _) = setup().await;
    invalid(
        repo.items()
            .create(ResourceKind::Thing, "Tent", 0, "", "", "")
            .await,
    );
    let tent = make(&repo, ResourceKind::Thing, 5).await;
    invalid(repo.items().set_quantity(tent.id, 5, 0).await);
    invalid(
        repo.reservations()
            .request(tent.id, 0, t(0), t(1), ME, "", "")
            .await,
    );
    let r = hold(&repo, &tent, 2, 0, 5).await;
    invalid(repo.reservations().change(r.id, 0, t(0), t(5)).await);
    invalid(repo.reservations().finish(r.id, 0).await);
    invalid(repo.needs().create("tents", 0, "", None, None, "").await);
    let need = repo
        .needs()
        .create("tents", 1, "", None, None, "")
        .await
        .unwrap();
    invalid(repo.needs().offer(need.id, ME, 0, None, "").await);
    invalid(
        repo.capacity()
            .propose(&person("a"), "", t(0), t(1), 0)
            .await,
    );
}

#[tokio::test]
async fn zero_duration_and_backwards_ranges_are_refused() {
    let (repo, _) = setup().await;
    let tent = make(&repo, ResourceKind::Thing, 5).await;
    for (from, to) in [(5, 5), (6, 5)] {
        invalid(
            repo.reservations()
                .request(tent.id, 1, t(from), t(to), ME, "", "")
                .await,
        );
        invalid(repo.reservations().free(tent.id, t(from), t(to)).await);
        invalid(
            repo.needs()
                .create("x", 1, "", Some(t(from)), Some(t(to)), "")
                .await,
        );
        invalid(repo.capacity().mine(t(from), t(to)).await);
        invalid(repo.capacity().load(t(from), t(to)).await);
        invalid(repo.capacity().allocations(t(from), t(to)).await);
        invalid(
            repo.capacity()
                .propose(&person("a"), "", t(from), t(to), 60)
                .await,
        );
        let c = Capacity {
            id: ObjectId::new(),
            person: ME.into(),
            from: t(from),
            to: t(to),
            minutes_per_week: 60,
            note: String::new(),
        };
        invalid(repo.capacity().set(c).await);
    }
    let r = hold(&repo, &tent, 1, 0, 5).await;
    invalid(repo.reservations().change(r.id, 1, t(5), t(5)).await);
    invalid(repo.reservations().change(r.id, 1, t(6), t(5)).await);
    // nothing changed
    let same = repo.reservations().get(r.id).await.unwrap().unwrap();
    assert_eq!(same, r);
}

#[tokio::test]
async fn a_need_with_only_one_end_of_a_time_is_refused() {
    let (repo, _) = setup().await;
    invalid(repo.needs().create("x", 1, "", Some(t(0)), None, "").await);
    invalid(repo.needs().create("x", 1, "", None, Some(t(1)), "").await);
}

#[tokio::test]
async fn a_blank_name_or_holder_is_refused() {
    let (repo, _) = setup().await;
    invalid(
        repo.items()
            .create(ResourceKind::Thing, "  ", 1, "", "", "")
            .await,
    );
    let tent = make(&repo, ResourceKind::Thing, 5).await;
    invalid(
        repo.reservations()
            .request(tent.id, 1, t(0), t(1), "", "", "")
            .await,
    );
    invalid(repo.needs().create(" ", 1, "", None, None, "").await);
}

#[tokio::test]
async fn delete_is_refused_while_a_confirmed_reservation_exists() {
    let (repo, _) = setup().await;
    let tent = make(&repo, ResourceKind::Thing, 5).await;
    let waiting = ask(&repo, &tent, 1, 20, 30).await;
    let r = hold(&repo, &tent, 2, 0, 10).await;
    let err = repo.items().delete(tent.id).await.unwrap_err();
    assert!(matches!(err, Error::Conflict(_)));
    assert!(repo.items().get(tent.id).await.unwrap().is_some());
    repo.reservations().cancel(r.id).await.unwrap();
    repo.items().delete(tent.id).await.unwrap();
    assert!(repo.items().get(tent.id).await.unwrap().is_none());
    // the rest that hung on it went too
    assert!(repo.reservations().get(waiting.id).await.unwrap().is_none());
    assert!(repo.reservations().get(r.id).await.unwrap().is_none());
}

#[tokio::test]
async fn unknown_ids_are_not_found() {
    let (repo, _) = setup().await;
    let nobody = ObjectId::new();
    assert!(matches!(
        repo.reservations().confirm(nobody).await,
        Err(Error::NotFound { .. })
    ));
    assert!(matches!(
        repo.reservations()
            .request(nobody, 1, t(0), t(1), ME, "", "")
            .await,
        Err(Error::NotFound { .. })
    ));
    assert!(repo.items().get(nobody).await.unwrap().is_none());
}
