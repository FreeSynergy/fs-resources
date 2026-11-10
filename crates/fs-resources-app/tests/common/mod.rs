//! Shared by the backend tests: the real core over in-memory stores, no test double.
#![allow(dead_code)] // each test file uses a different subset

use std::sync::Arc;

use fs_admin_frontend::{Backend, Record, Values};
use fs_objects::TursoStore;
use fs_resources_app::{CoreBackend, Ports};
use fs_resources_core::ResourceRepo;

pub struct World {
    pub rt: tokio::runtime::Runtime,
    pub ports: Ports,
}

impl World {
    pub fn new() -> Self {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let repo = rt.block_on(async {
            let main = Arc::new(TursoStore::open_in_memory().await.unwrap());
            let private = Arc::new(TursoStore::open_in_memory().await.unwrap());
            ResourceRepo::new(main, private, "https://example.org/people/me")
                .with_instance("example.org")
        });
        Self {
            ports: Ports::from_repo(&repo),
            rt,
        }
    }

    pub fn of(&self, data_type: &'static str) -> CoreBackend {
        CoreBackend::new(data_type, self.ports.clone(), self.rt.handle().clone())
    }
}

pub fn v(pairs: &[(&str, &[&str])]) -> Values {
    pairs
        .iter()
        .map(|(k, vs)| {
            (
                (*k).to_owned(),
                vs.iter().map(|s| (*s).to_owned()).collect(),
            )
        })
        .collect()
}

pub fn first(r: &Record, field: &str) -> String {
    r.values
        .get(field)
        .and_then(|x| x.first())
        .cloned()
        .unwrap_or_default()
}

pub fn made(b: &CoreBackend, name: &str, input: &Values) -> String {
    b.run(name, None, input)
        .unwrap()
        .id
        .expect("the command names what it made")
}

pub const T0: &str = "2026-03-02T09:00:00Z";
pub const T1: &str = "2026-03-02T17:00:00Z";
pub const T2: &str = "2026-03-03T09:00:00Z";
