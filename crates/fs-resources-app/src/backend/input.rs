//! Reading the typed input of a command, and the refusals that come from it.
//!
//! Pattern: Adapter (text of the form to the types of the core) — the frontend checked every value
//! against its basic type already; what is left is the step from text to `u32`, `Timestamp` and
//! `ObjectId`. A text that still does not fit is refused as a finding on that field.

use chrono::{DateTime, SecondsFormat, Utc};
use fs_admin_frontend::{BackendError, Values};
use fs_objects::{ObjectId, Timestamp};
use fs_types::{RecordIssue, RecordProblem, TypeProblem};

/// The values of one command, read by field name.
pub struct Input<'a>(pub &'a Values);

/// Why the input cannot be used: a finding on `field`.
pub fn bad(field: &str) -> BackendError {
    BackendError::Rejected(vec![RecordIssue {
        field: field.to_owned(),
        index: 0,
        problem: RecordProblem::Value(TypeProblem::Character),
    }])
}

impl Input<'_> {
    /// Whether the field was sent at all (an empty list is "cleared", a missing field "unchanged").
    pub fn has(&self, name: &str) -> bool {
        self.0.contains_key(name)
    }

    /// Refuse a `change` that names a field the core cannot change (state, resource, holder ...): a finding on it,
    /// so an edit is never dropped silently.
    pub fn only(&self, changeable: &[&str]) -> Result<(), BackendError> {
        let refused: Vec<RecordIssue> = self
            .0
            .keys()
            .filter(|k| k.as_str() != "id" && !changeable.contains(&k.as_str()))
            .map(|k| RecordIssue {
                field: k.clone(),
                index: 0,
                problem: RecordProblem::UnknownField,
            })
            .collect();
        if refused.is_empty() {
            Ok(())
        } else {
            Err(BackendError::Rejected(refused))
        }
    }

    /// The first non-empty value.
    pub fn text(&self, name: &str) -> Option<&str> {
        self.0
            .get(name)?
            .iter()
            .map(|t| t.trim())
            .find(|t| !t.is_empty())
    }

    /// The text, or empty.
    pub fn text_or_empty(&self, name: &str) -> &str {
        self.text(name).unwrap_or("")
    }

    /// A number; none if the field is missing or empty.
    pub fn number(&self, name: &str) -> Result<Option<u32>, BackendError> {
        self.text(name)
            .map(|t| t.parse().map_err(|_| bad(name)))
            .transpose()
    }

    /// A number that must be there.
    pub fn need_number(&self, name: &str) -> Result<u32, BackendError> {
        self.number(name)?.ok_or_else(|| bad(name))
    }

    /// A time (RFC 3339); none if the field is missing or empty.
    pub fn time(&self, name: &str) -> Result<Option<Timestamp>, BackendError> {
        self.text(name)
            .map(|t| {
                DateTime::parse_from_rfc3339(t)
                    .map(|d| d.with_timezone(&Utc))
                    .map_err(|_| bad(name))
            })
            .transpose()
    }

    /// A time that must be there.
    pub fn need_time(&self, name: &str) -> Result<Timestamp, BackendError> {
        self.time(name)?.ok_or_else(|| bad(name))
    }

    /// The id in a reference: an IRI whose last step is the id, a `urn:uuid:` or the bare id.
    pub fn id(&self, name: &str) -> Result<Option<ObjectId>, BackendError> {
        self.text(name)
            .map(|t| object_id(t).ok_or_else(|| bad(name)))
            .transpose()
    }

    /// An id that must be there.
    pub fn need_id(&self, name: &str) -> Result<ObjectId, BackendError> {
        self.id(name)?.ok_or_else(|| bad(name))
    }

    /// Keywords sent as one comma-separated text.
    pub fn words(&self, name: &str) -> Vec<String> {
        self.text(name)
            .map(|t| {
                t.split(',')
                    .map(|w| w.trim().to_owned())
                    .filter(|w| !w.is_empty())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Whether a choice field holds `key`.
    pub fn is(&self, name: &str, key: &str) -> bool {
        self.text(name) == Some(key)
    }
}

/// The id at the end of an IRI (or the id itself).
pub fn object_id(text: &str) -> Option<ObjectId> {
    let last = text.rsplit('/').next().unwrap_or(text);
    last.parse().ok()
}

/// A time as it is shown and stored in a record: RFC 3339, UTC.
pub fn stamp(t: Timestamp) -> String {
    t.to_rfc3339_opts(SecondsFormat::Secs, true)
}

/// The earliest and latest time a list without a range covers.
pub fn forever() -> (Timestamp, Timestamp) {
    (
        Timestamp::from_timestamp(0, 0).unwrap_or_default(),
        Timestamp::from_timestamp(4_102_444_800, 0).unwrap_or_default(),
    )
}
