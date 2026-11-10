//! A real call: the type catalog and the reports over a real Unix socket, in front of the real core.
//!
//! Pattern: none (tests). The server is exercised only through the generated client.

use std::sync::Arc;

use fs_interfaces::resources_capnp::resources;
use fs_interfaces::rpc::{bind_socket, connect, serve};
use fs_interfaces::types_capnp::{BasicType, CommandKind};
use fs_objects::TursoStore;
use fs_resources_core::ResourceRepo;
use fs_resources_core::describe::COMMANDS;
use fs_resources_server::ResourcesServer;

async fn with_root<F, Fut>(test: F)
where
    F: FnOnce(resources::Client) -> Fut,
    Fut: Future<Output = ()>,
{
    let local = tokio::task::LocalSet::new();
    local
        .run_until(async {
            let main = Arc::new(TursoStore::open_in_memory().await.unwrap());
            let private = Arc::new(TursoStore::open_in_memory().await.unwrap());
            let repo = ResourceRepo::new(main, private, "https://example.org/people/me");
            let root = ResourcesServer::new(Arc::new(repo)).into_client();
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("resources.sock");
            let bound = bind_socket(&path).await.unwrap();
            let (stop, stopped) = tokio::sync::oneshot::channel::<()>();
            let serving = tokio::task::spawn_local(serve(bound, root.client, async {
                let _ = stopped.await;
            }));
            let connection = connect::<resources::Client>(&path).await.unwrap();
            test(connection.client.clone()).await;
            drop(connection);
            let _ = stop.send(());
            serving.await.unwrap().unwrap();
        })
        .await;
}

#[tokio::test(flavor = "current_thread")]
async fn types_answers_six_types_with_all_commands_missing_ones_included() {
    with_root(|root| async move {
        let answer = root.types_request().send().promise.await.unwrap();
        let catalog = answer.get().unwrap().get_catalog().unwrap();
        let answer = catalog.describe_request().send().promise.await.unwrap();
        let types = answer.get().unwrap().get_types().unwrap();
        let names: Vec<String> = types
            .iter()
            .map(|t| t.get_name().unwrap().to_string().unwrap())
            .collect();
        assert_eq!(
            names,
            [
                "Allocation",
                "Capacity",
                "Need",
                "Offer",
                "Reservation",
                "Resource"
            ]
        );
        let mut commands = 0;
        for t in types.iter() {
            assert_eq!(t.get_program().unwrap().to_str().unwrap(), "fs-resources");
            commands += t.get_commands().unwrap().len() as usize;
        }
        assert_eq!(commands, COMMANDS.len());
        let resource = types
            .iter()
            .find(|t| t.get_name().unwrap() == "Resource")
            .unwrap();
        let kind = resource
            .get_fields()
            .unwrap()
            .iter()
            .find(|f| f.get_name().unwrap() == "kind")
            .unwrap();
        assert_eq!(kind.get_basic().unwrap(), BasicType::Choice);
        let missing = resource
            .get_commands()
            .unwrap()
            .iter()
            .find(|c| c.get_name().unwrap() == "setMinimum")
            .expect("a command a requirement asks for is in the description");
        assert_eq!(missing.get_kind().unwrap(), CommandKind::Other);
        assert!(
            !missing.get_supported(),
            "missing commands travel with supported = false"
        );
        let create = resource
            .get_commands()
            .unwrap()
            .iter()
            .find(|c| c.get_name().unwrap() == "create")
            .unwrap();
        assert!(create.get_supported());
    })
    .await;
}

#[tokio::test(flavor = "current_thread")]
async fn the_reports_come_from_the_core() {
    with_root(|root| async move {
        let answer = root.support_report_request().send().promise.await.unwrap();
        let report = answer.get().unwrap().get_report().unwrap();
        assert_eq!(
            report.get_provider().unwrap().to_str().unwrap(),
            "fs-resources-core"
        );
        let entries = report.get_entries().unwrap();
        assert!(
            entries
                .iter()
                .any(|e| e.get_capability().unwrap() == "resource.create" && e.get_supported())
        );
        let answer = root.capabilities_request().send().promise.await.unwrap();
        let ids = answer
            .get()
            .unwrap()
            .get_capabilities()
            .unwrap()
            .get_ids()
            .unwrap();
        assert!(ids.iter().any(|i| i.unwrap() == "reservation.decide"));
    })
    .await;
}

#[tokio::test(flavor = "current_thread")]
async fn the_rest_of_the_contract_is_not_served() {
    with_root(|root| async move {
        let refused = root.items_request().send().promise.await;
        assert!(
            refused.is_err(),
            "items must answer unimplemented, not an empty capability"
        );
        let refused = root.capacity_request().send().promise.await;
        assert!(
            refused.is_err(),
            "capacity stays refused until the transport knows the caller"
        );
    })
    .await;
}
