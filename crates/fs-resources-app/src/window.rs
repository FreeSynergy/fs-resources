//! The window: one screen per data type, made by the generated frontend.
//!
//! Pattern: Facade — builds the `Session` of each type from its description and a [`CoreBackend`],
//! and hands the views to `admin_window`. The order of the screens is the order of the registry.

use std::sync::Arc;

use fs_admin_frontend::view::{AboutInfo, AdminSurface};
use fs_admin_frontend::window::{TypeView, admin_window};
use fs_admin_frontend::{Session, SystemClock};
use fs_components_app::{AppError, AppWidget};
use fs_components_fields::{Look, Say};
use fs_components_styles::StyleId;
use fs_resources_core::describe::registry;
use tokio::runtime::Handle;

use crate::backend::{CoreBackend, Ports};
use crate::words::{TITLE, Words};

/// Where the installed program came from, in the About page. Provisional: the store will say it.
pub const ABOUT_ORIGIN: &str = "FreeSynergy catalog";

/// A screen per data type of the program.
///
/// # Panics
/// Never for the shipped tables: their descriptions are checked by the tests of the core.
pub fn views(ports: &Ports, runtime: &Handle, say: &Arc<dyn Say>) -> Vec<TypeView> {
    let registry = registry().expect("the descriptions of fs-resources are sound");
    registry
        .iter()
        .map(|d| {
            let name: &'static str = leak(&d.name);
            let backend = Arc::new(CoreBackend::new(name, ports.clone(), runtime.clone()));
            let session = Session::new(d.clone(), backend, Arc::new(SystemClock::new()));
            TypeView {
                id: d.name.clone(),
                label: d.label.clone(),
                surface: Arc::new(AdminSurface::new(
                    session,
                    Look::new(StyleId::Admin, false),
                    Arc::clone(say),
                    None,
                )),
            }
        })
        .collect()
}

/// The names of the types are the `'static` ones of the tables; find the table's own copy.
fn leak(name: &str) -> &'static str {
    use fs_resources_core::describe::{ALLOCATION, CAPACITY, NEED, OFFER, RESERVATION, RESOURCE};
    [RESOURCE, RESERVATION, NEED, OFFER, CAPACITY, ALLOCATION]
        .into_iter()
        .find(|n| *n == name)
        .unwrap_or(RESOURCE)
}

/// The shell around the screens.
///
/// # Errors
/// [`AppError`] if the shell cannot be made.
pub fn app_widget(ports: &Ports, runtime: &Handle) -> Result<AppWidget, AppError> {
    let say: Arc<dyn Say> = Arc::new(Words);
    admin_window(
        TITLE,
        &say,
        views(ports, runtime, &say),
        AboutInfo {
            name: TITLE.to_owned(),
            version: format!("v{}", env!("CARGO_PKG_VERSION")),
            origin: ABOUT_ORIGIN.to_owned(),
            licence: env!("CARGO_PKG_LICENSE").to_owned(),
            repository: "https://github.com/FreeSynergy/fs-resources".to_owned(),
        },
        "https://freesynergy.github.io/fs-documentation/".to_owned(),
    )
}
