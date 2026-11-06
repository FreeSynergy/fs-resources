//! Private records live in a store of their own; the main store never holds one.

mod common;

use common::{cap, person, setup, t};
use fs_store::traits::{ObjectQuery, UnloggedObjects};
use fs_store::{ObjectId, Query, Store, TursoStore};

const WEEK: i64 = 168;

async fn count(s: &TursoStore, ty: &'static str) -> usize {
    ObjectQuery::count(s, &Query::new().of_type(ty))
        .await
        .unwrap()
}

#[tokio::test]
async fn private_records_live_only_in_the_private_store() {
    let (a, store) = setup().await;
    let c = a.capacity().set(cap(600, 0, WEEK)).await.unwrap();
    let ask = a
        .capacity()
        .propose(&person("z"), "t", t(0), t(WEEK), 60)
        .await
        .unwrap();
    a.capacity()
        .share_with_group(ObjectId::new(), true)
        .await
        .unwrap();
    for ty in ["Capacity", "Allocation", "GroupShare"] {
        assert_eq!(count(&store.main, ty).await, 0, "{ty} in the main store");
        assert_eq!(
            count(&store.private, ty).await,
            1,
            "{ty} in the private store"
        );
    }
    for id in [c.id, ask.id] {
        assert!(Store::get(&*store.main, id).await.unwrap().is_none());
        assert!(store.private.read_unlogged(id).await.unwrap().is_some());
    }
    assert_eq!(store.main.log_stats().await.unwrap().entries, 0);
}
