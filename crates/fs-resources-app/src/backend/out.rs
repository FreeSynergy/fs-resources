//! Values going out: a record or a result as the frontend wants it.
//!
//! Pattern: Builder — `Vals::new().put("name", ..)` skips empty values, so a field that is not set
//! is absent, not an empty text.

use fs_admin_frontend::{Outcome, Record, Values};

/// A set of values under construction.
#[derive(Default)]
pub struct Vals(Values);

impl Vals {
    /// Nothing yet.
    pub fn new() -> Self {
        Self::default()
    }

    /// One value; nothing if the text is empty.
    pub fn put(mut self, name: &str, text: impl Into<String>) -> Self {
        let text = text.into();
        if !text.trim().is_empty() {
            self.0.entry(name.to_owned()).or_default().push(text);
        }
        self
    }

    /// One more value of a list field (kept in order).
    pub fn push(self, name: &str, text: impl Into<String>) -> Self {
        self.put(name, text)
    }

    /// A number.
    pub fn num<N: Into<u64>>(self, name: &str, n: N) -> Self {
        self.put(name, n.into().to_string())
    }

    /// An optional text.
    pub fn opt(self, name: &str, text: Option<String>) -> Self {
        match text {
            Some(t) => self.put(name, t),
            None => self,
        }
    }

    /// As a record with this id.
    pub fn record(self, id: String) -> Record {
        Record { id, values: self.0 }
    }

    /// As the result of a command.
    pub fn outcome(self) -> Outcome {
        Outcome {
            id: None,
            values: self.0,
        }
    }
}

/// The result of a command that made or changed the record `id`.
pub fn made(id: String) -> Outcome {
    Outcome {
        id: Some(id),
        values: Values::new(),
    }
}

/// The result of a command with nothing to tell.
pub fn done() -> Outcome {
    Outcome::default()
}
