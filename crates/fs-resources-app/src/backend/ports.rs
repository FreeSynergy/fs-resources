//! The four contract traits of the core, as the backend uses them.
//!
//! Pattern: Dependency inversion (coding.md section 1) — the backend depends on `Items`,
//! `Reservations`, `Capacities` and `Needs`, never on `ResourceRepo`. Today they are all the repo in
//! this process; a Cap'n Proto client that implements the same traits goes in here without a change
//! to the backend.

use std::sync::Arc;

use fs_objects::ObjectId;
use fs_resources_core::{Capacities, Items, Needs, Reservations, ResourceRepo};

/// The traits the backend calls and the way an object gets its IRI (the AS2 `id`).
#[derive(Clone)]
pub struct Ports {
    /// Things, rooms and material.
    pub items: Arc<dyn Items>,
    /// Reservations.
    pub reservations: Arc<dyn Reservations>,
    /// The acting person's own time.
    pub capacities: Arc<dyn Capacities>,
    /// Needs and offers.
    pub needs: Arc<dyn Needs>,
    iri: Arc<dyn Fn(ObjectId) -> String + Send + Sync>,
}

impl Ports {
    /// All four from one repository.
    pub fn from_repo(repo: &ResourceRepo) -> Self {
        let for_iri = repo.clone();
        Self {
            items: Arc::new(repo.clone()),
            reservations: Arc::new(repo.clone()),
            capacities: Arc::new(repo.clone()),
            needs: Arc::new(repo.clone()),
            iri: Arc::new(move |id| for_iri.iri_of(id)),
        }
    }

    /// The IRI of an object.
    pub fn iri(&self, id: ObjectId) -> String {
        (self.iri)(id)
    }
}
