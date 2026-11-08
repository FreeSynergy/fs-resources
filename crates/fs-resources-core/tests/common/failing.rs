//! A store that fails on the n-th write, to show which state a crash would leave.

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use async_trait::async_trait;
use fs_objects::traits::{ChangeFeed, ObjectQuery, ObjectRepository, UnloggedObjects};
use fs_objects::{Activity, Error, LogEntry, ObjectId, Query, Result, StoredObject, TursoStore};
use tokio::sync::broadcast;

/// Wraps a real store; every write is counted and the chosen one fails before it reaches the store.
pub struct Failing {
    inner: Arc<TursoStore>,
    writes: AtomicUsize,
    fail_on: AtomicUsize,
}

impl Failing {
    pub fn new(inner: Arc<TursoStore>) -> Arc<Self> {
        Arc::new(Self {
            inner,
            writes: AtomicUsize::new(0),
            fail_on: AtomicUsize::new(0),
        })
    }

    /// From now on, the `n`-th write (1 = the next one) fails; the ones before it go through.
    pub fn fail_nth_write(&self, n: usize) {
        self.writes.store(0, Ordering::SeqCst);
        self.fail_on.store(n, Ordering::SeqCst);
    }

    fn write(&self) -> Result<()> {
        let n = self.writes.fetch_add(1, Ordering::SeqCst) + 1;
        if n == self.fail_on.load(Ordering::SeqCst) {
            return Err(Error::Backend("write fault".into()));
        }
        Ok(())
    }
}

#[async_trait]
impl ObjectRepository for Failing {
    async fn put(&self, object: StoredObject, actor: &str) -> Result<Activity> {
        self.write()?;
        ObjectRepository::put(&*self.inner, object, actor).await
    }
    async fn put_and_log(
        &self,
        object: StoredObject,
        actor: &str,
        extra: Vec<LogEntry>,
    ) -> Result<Vec<Activity>> {
        self.write()?;
        ObjectRepository::put_and_log(&*self.inner, object, actor, extra).await
    }
    async fn get(&self, id: ObjectId) -> Result<Option<StoredObject>> {
        ObjectRepository::get(&*self.inner, id).await
    }
    async fn delete(&self, id: ObjectId, actor: &str) -> Result<Activity> {
        self.write()?;
        ObjectRepository::delete(&*self.inner, id, actor).await
    }
}

#[async_trait]
impl ObjectQuery for Failing {
    async fn query(&self, query: &Query) -> Result<Vec<StoredObject>> {
        ObjectQuery::query(&*self.inner, query).await
    }
    async fn count(&self, query: &Query) -> Result<usize> {
        ObjectQuery::count(&*self.inner, query).await
    }
}

impl ChangeFeed for Failing {
    fn subscribe(&self) -> broadcast::Receiver<Activity> {
        ChangeFeed::subscribe(&*self.inner)
    }
}

#[async_trait]
impl UnloggedObjects for Failing {
    async fn put_unlogged(&self, object: StoredObject) -> Result<()> {
        self.write()?;
        self.inner.put_unlogged(object).await
    }
    async fn read_unlogged(&self, id: ObjectId) -> Result<Option<StoredObject>> {
        self.inner.read_unlogged(id).await
    }
    async fn delete_unlogged(&self, id: ObjectId) -> Result<()> {
        self.write()?;
        self.inner.delete_unlogged(id).await
    }
}
