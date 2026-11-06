//! Items: create, edit, list with filters and paging.

mod common;

use common::{ME, make, person, setup};
use fs_resources_core::{Cursor, Error, ResourceFilter, ResourceKind};

#[tokio::test]
async fn create_get_and_edit() {
    let (repo, _) = setup().await;
    let items = repo.items();
    let r = items
        .create(
            ResourceKind::Thing,
            " Projector ",
            2,
            "piece",
            &person("a"),
            "https://example.org/places/hall",
        )
        .await
        .unwrap();
    assert_eq!(r.name, "Projector");
    assert_eq!(items.get(r.id).await.unwrap().unwrap(), r);
    let labels = vec!["av".to_string(), String::new(), "loan".to_string()];
    let e = items
        .edit(r.id, "Beamer", "pc", "", "", &labels)
        .await
        .unwrap();
    assert_eq!(
        (e.name.as_str(), e.owner.as_str(), e.labels.len()),
        ("Beamer", "", 2)
    );
    assert_eq!(e.kind, ResourceKind::Thing);
    assert_eq!(e.quantity, 2);
    assert!(matches!(
        items.edit(r.id, "", "", "", "", &[]).await,
        Err(Error::Invalid(_))
    ));
}

#[tokio::test]
async fn the_iri_and_envelope_follow_the_program_rules() {
    let (repo, store) = setup().await;
    let r = make(&repo, ResourceKind::Thing, 1).await;
    use fs_store::Store;
    let o = store.get(r.id).await.unwrap().unwrap();
    assert_eq!(o.envelope.object_type, "Resource");
    assert_eq!(o.envelope.context.as_deref(), Some("resources"));
    assert_eq!(
        o.envelope.iri.unwrap(),
        format!("https://example.org/resources/{}", r.id)
    );
    assert_eq!(o.envelope.attributed_to.as_deref(), Some(ME));
    assert_eq!(o.content["fs:quantity"], 1);
    assert_eq!(o.content["type"], "Resource");
}

#[tokio::test]
async fn list_filters_by_kind_owner_place_label_and_name() {
    let (repo, _) = setup().await;
    let items = repo.items();
    let a = items
        .create(ResourceKind::Thing, "Red tent", 3, "", &person("a"), "p1")
        .await
        .unwrap();
    items
        .edit(
            a.id,
            "Red tent",
            "",
            &person("a"),
            "p1",
            &["camp".to_string()],
        )
        .await
        .unwrap();
    items
        .create(ResourceKind::Room, "Hall", 1, "", &person("b"), "p2")
        .await
        .unwrap();
    items
        .create(
            ResourceKind::Material,
            "Paint",
            9,
            "litre",
            &person("a"),
            "",
        )
        .await
        .unwrap();
    let names = |f: ResourceFilter| {
        let items = repo.clone();
        async move {
            let p = items.items().list(&f, None).await.unwrap();
            p.items.into_iter().map(|r| r.name).collect::<Vec<_>>()
        }
    };
    assert_eq!(
        names(ResourceFilter::default()).await,
        ["Hall", "Paint", "Red tent"]
    );
    assert_eq!(
        names(ResourceFilter {
            kind: Some(ResourceKind::Room),
            ..Default::default()
        })
        .await,
        ["Hall"]
    );
    assert_eq!(
        names(ResourceFilter {
            owner: person("a"),
            ..Default::default()
        })
        .await,
        ["Paint", "Red tent"]
    );
    assert_eq!(
        names(ResourceFilter {
            place: "p2".into(),
            ..Default::default()
        })
        .await,
        ["Hall"]
    );
    assert_eq!(
        names(ResourceFilter {
            label: "camp".into(),
            ..Default::default()
        })
        .await,
        ["Red tent"]
    );
    assert_eq!(
        names(ResourceFilter {
            name: "TENT".into(),
            ..Default::default()
        })
        .await,
        ["Red tent"]
    );
}

#[tokio::test]
async fn list_pages_with_a_cursor() {
    let (repo, _) = setup().await;
    let repo = repo.with_page_size(2);
    for n in ["a", "b", "c", "d", "e"] {
        repo.items()
            .create(ResourceKind::Thing, n, 1, "", "", "")
            .await
            .unwrap();
    }
    let mut seen = vec![];
    let mut cursor: Option<Cursor> = None;
    let mut pages = 0;
    loop {
        let page = repo
            .items()
            .list(&ResourceFilter::default(), cursor.as_ref())
            .await
            .unwrap();
        seen.extend(page.items.into_iter().map(|r| r.name));
        pages += 1;
        match page.next {
            Some(c) => cursor = Some(c),
            None => break,
        }
    }
    assert_eq!((seen.join(""), pages), ("abcde".to_string(), 3));
}
