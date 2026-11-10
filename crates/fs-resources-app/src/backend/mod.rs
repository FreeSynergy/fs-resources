//! The `Backend` of the admin frontend over the resources core.
//!
//! Pattern: Repository (the frontend's trait, ADR 0043) with Strategy by data type — [`CoreBackend`]
//! is one value per data type; it refuses every command the table marks unsupported before it looks
//! at the input, then hands the rest to the module of its type. The async core is called on the
//! runtime handle: the frontend's `Backend` is synchronous (its DESIGN.md says so).

mod allocation;
mod capacity;
mod errors;
mod input;
mod need;
mod offer;
mod out;
mod ports;
mod reservation;
mod resource;

use fs_admin_frontend::{Backend, BackendError, Outcome, Record, Values};
use fs_resources_core::describe::{
    ALLOCATION, CAPACITY, NEED, OFFER, RESERVATION, RESOURCE, command, is_unsupported,
};
use fs_types::describe::Edit;
use fs_types::{RecordIssue, RecordProblem};
use tokio::runtime::Handle;

use input::Input;
pub use ports::Ports;

/// The records and commands of one data type of the program.
pub struct CoreBackend {
    data_type: &'static str,
    ports: Ports,
    runtime: Handle,
}

impl CoreBackend {
    /// The backend of `data_type` (one of the names in `describe`).
    pub fn new(data_type: &'static str, ports: Ports, runtime: Handle) -> Self {
        Self {
            data_type,
            ports,
            runtime,
        }
    }
}

/// The refusal for a command that is described but not built: a finding on the command's own name.
pub fn unsupported_refusal(name: &str) -> BackendError {
    BackendError::Rejected(vec![RecordIssue {
        field: name.to_owned(),
        index: 0,
        problem: RecordProblem::UnknownField,
    }])
}

impl Backend for CoreBackend {
    fn list(&self, query: &Values) -> Result<Vec<Record>, BackendError> {
        let (p, q) = (&self.ports, Input(query));
        self.runtime.block_on(async {
            match self.data_type {
                RESOURCE => resource::list(p, &q).await,
                RESERVATION => reservation::list(p, &q).await,
                NEED => need::list(p, &q).await,
                OFFER => offer::list(p, &q).await,
                CAPACITY => capacity::list(p, &q).await,
                ALLOCATION => allocation::list(p, &q).await,
                _ => Err(BackendError::NotFound),
            }
        })
    }

    fn supported(&self, name: &str) -> bool {
        command(self.data_type, name).is_some_and(|c| c.supported)
    }

    fn change(&self, record: &str, edit: &Edit) -> Result<Outcome, BackendError> {
        // The edit says what to change field by field; the rule (missing = unchanged, empty = cleared) is the
        // one `run("change")` of every module reads from `Input::has`.
        let mut values = Values::new();
        for f in edit.fields.iter().filter(|f| f.path.len() == 1) {
            values.insert(f.path[0].clone(), f.values.clone());
        }
        self.run("change", Some(record), &values)
    }

    fn run(
        &self,
        name: &str,
        record: Option<&str>,
        input: &Values,
    ) -> Result<Outcome, BackendError> {
        if command(self.data_type, name).is_none() && name != "change" {
            return Err(BackendError::NotFound);
        }
        if is_unsupported(self.data_type, name) {
            return Err(unsupported_refusal(name));
        }
        // The record travels next to the input; the modules read it as the field `id`.
        let mut values = input.clone();
        if let Some(record) = record {
            values.insert("id".to_owned(), vec![record.to_owned()]);
        }
        let (p, i) = (&self.ports, Input(&values));
        self.runtime.block_on(async {
            match self.data_type {
                RESOURCE => resource::run(p, name, &i).await,
                RESERVATION => reservation::run(p, name, &i).await,
                NEED => need::run(p, name, &i).await,
                OFFER => offer::run(p, name, &i).await,
                CAPACITY => capacity::run(p, name, &i).await,
                ALLOCATION => allocation::run(p, name, &i).await,
                _ => Err(BackendError::NotFound),
            }
        })
    }
}
