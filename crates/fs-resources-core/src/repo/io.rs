//! Typed storage: save, load, find and erase any [`Record`].
//!
//! Pattern: Repository (generic) + Adapter — one place knows how a value becomes an envelope with
//! index fields. Public records go through the logged store; private ones through
//! `UnloggedObjects`, with no actor and a fixed time, and are read back the same way.

use fs_objects::{Envelope, ObjectId, Query, StoredObject};

use super::consts::{AS2_TYPE, CONTEXT, EPOCH};
use super::handle::ResourceRepo;
use crate::error::Result;
use crate::model::{Record, document, from_document};

impl ResourceRepo {
    pub(super) fn iri(&self, id: ObjectId) -> String {
        format!("https://{}/resources/{}", self.instance, id)
    }

    fn object<T: Record>(&self, value: &T) -> Result<StoredObject> {
        let mut e = Envelope::new(T::TYPE, value.name());
        e.id = value.id();
        e.iri = Some(self.iri(e.id));
        e.context = Some(CONTEXT.into());
        if T::PRIVATE {
            e.published = EPOCH;
            e.updated = EPOCH;
        } else {
            e.attributed_to = Some(self.actor.clone());
            e.to = vec![self.actor.clone()];
        }
        let doc = document(value, &self.iri(e.id))?;
        let mut o = StoredObject::new(e, AS2_TYPE, doc);
        for (key, v) in value.index() {
            if !v.is_empty() {
                o = o.with_index(key, v);
            }
        }
        Ok(o)
    }

    /// Create or replace.
    pub(super) async fn save<T: Record>(&self, value: &T) -> Result<()> {
        let o = self.object(value)?;
        if T::PRIVATE {
            self.private.put_unlogged(o).await?;
        } else {
            self.objects.put(o, &self.actor).await?;
        }
        Ok(())
    }

    /// One value by id, if it is there and of this type.
    pub(super) async fn load<T: Record>(&self, id: ObjectId) -> Result<Option<T>> {
        let o = if T::PRIVATE {
            self.private.read_unlogged(id).await?
        } else {
            self.objects.get(id).await?
        };
        Ok(o.filter(|o| is_ours::<T>(o))
            .and_then(|o| from_document(&o.content)))
    }

    /// The value, or `NotFound`.
    pub(super) async fn require<T: Record>(&self, what: &'static str, id: ObjectId) -> Result<T> {
        self.load(id)
            .await?
            .ok_or(crate::Error::NotFound { what, id })
    }

    /// All values of this type matching `q` (type and context are added).
    pub(super) async fn find<T: Record>(&self, q: Query) -> Result<Vec<T>> {
        let q = q.of_type(T::TYPE).in_context(CONTEXT);
        let found = if T::PRIVATE {
            self.private.query_unlogged(&q, &[T::TYPE]).await?
        } else {
            self.reads.query(&q).await?
        };
        Ok(found
            .iter()
            .filter_map(|o| from_document(&o.content))
            .collect())
    }

    /// Delete one value, if it is there.
    pub(super) async fn erase<T: Record>(&self, id: ObjectId) -> Result<()> {
        if T::PRIVATE {
            self.private.delete_unlogged(id).await?;
        } else if self.load::<T>(id).await?.is_some() {
            self.objects.delete(id, &self.actor).await?;
        }
        Ok(())
    }
}

fn is_ours<T: Record>(o: &StoredObject) -> bool {
    o.envelope.object_type == T::TYPE && o.envelope.context.as_deref() == Some(CONTEXT)
}
