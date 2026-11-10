//! The `Resources` root: support report, capability list and the type catalog.
//!
//! Pattern: Adapter — [`ResourcesServer`] is what a program serves; it depends on the core's
//! `Items` trait for the report and on the `TypeRegistry` for the catalog (ADR 0043), never on
//! `ResourceRepo`. The catalog is `fs_types::CatalogServer`, so the answer is the one every program
//! gives.

use std::rc::Rc;
use std::sync::Arc;

use capnp_rpc::new_client;
use fs_interfaces::resources_capnp::resources;
use fs_resources_core::Items;
use fs_resources_core::describe::registry;
use fs_types::describe::CatalogServer;

/// The version of `resources.capnp` this server answers.
pub const CONTRACT_VERSION: &str = "resources 0.1 (draft)";

/// The root capability of the resources program.
pub struct ResourcesServer {
    provider: Arc<dyn Items>,
}

impl ResourcesServer {
    /// Serve this provider.
    pub fn new(provider: Arc<dyn Items>) -> Self {
        Self { provider }
    }

    /// The capability to hand to `fs_interfaces::rpc::serve`.
    #[must_use]
    pub fn into_client(self) -> resources::Client {
        new_client(self)
    }
}

type Done = Result<(), capnp::Error>;

impl resources::Server for ResourcesServer {
    async fn capabilities(
        self: Rc<Self>,
        _: resources::CapabilitiesParams,
        mut results: resources::CapabilitiesResults,
    ) -> Done {
        let report = self.provider.capabilities();
        report.write_list(CONTRACT_VERSION, results.get().init_capabilities());
        Ok(())
    }

    async fn support_report(
        self: Rc<Self>,
        _: resources::SupportReportParams,
        mut results: resources::SupportReportResults,
    ) -> Done {
        let report = self.provider.capabilities();
        report.write(CONTRACT_VERSION, results.get().init_report());
        Ok(())
    }

    async fn types(
        self: Rc<Self>,
        _: resources::TypesParams,
        mut results: resources::TypesResults,
    ) -> Done {
        let registry =
            registry().map_err(|e| capnp::Error::failed(format!("types.unsound: {e:?}")))?;
        results
            .get()
            .set_catalog(new_client(CatalogServer::new(registry)));
        Ok(())
    }
}
