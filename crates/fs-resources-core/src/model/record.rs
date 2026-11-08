//! How a value is stored: the [`Record`] trait and the `fs:` content form.
//!
//! Pattern: Adapter — a typed value is written as an ActivityStreams-shaped document whose own
//! members live in the `fs:` namespace (no AS2 type exists for them), and read back from it. Each
//! value says its store type, its name, whether it is private, and its index fields.

use fs_objects::ObjectId;
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::{Map, Value, json};

/// Namespace of the members this program adds to AS2.
pub const NS: &str = "https://freesynergy.net/ns#";

/// A value that has a place in the object store.
pub trait Record: Serialize + DeserializeOwned {
    /// Store type of the object.
    const TYPE: &'static str;
    /// Private records are written with `UnloggedObjects`: no log, no event, no hub, no undo.
    const PRIVATE: bool = false;
    /// The id of the object.
    fn id(&self) -> ObjectId;
    /// Envelope name (ADR 0032); never secret.
    fn name(&self) -> String;
    /// Index fields for the filters; empty values are left out by the caller.
    fn index(&self) -> Vec<(&'static str, String)>;
}

/// The stored document of `value`: AS2 frame plus its fields as `fs:` members.
pub fn document<T: Record>(value: &T, iri: &str) -> serde_json::Result<Value> {
    let mut doc = Map::new();
    doc.insert(
        "@context".into(),
        json!(["https://www.w3.org/ns/activitystreams", {"fs": NS}]),
    );
    doc.insert("type".into(), json!(T::TYPE));
    doc.insert("id".into(), json!(iri));
    if let Value::Object(fields) = serde_json::to_value(value)? {
        for (k, v) in fields {
            doc.insert(format!("fs:{k}"), v);
        }
    }
    Ok(Value::Object(doc))
}

/// Read a value back from its document. `None` if it is not well formed.
pub fn from_document<T: Record>(doc: &Value) -> Option<T> {
    let fields: Map<String, Value> = doc
        .as_object()?
        .iter()
        .filter_map(|(k, v)| Some((k.strip_prefix("fs:")?.to_string(), v.clone())))
        .collect();
    serde_json::from_value(Value::Object(fields)).ok()
}
