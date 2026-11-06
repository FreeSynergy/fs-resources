//! The feed of public changes, taken from the store's change feed.
//!
//! Pattern: Observer + Adapter — one task per subscription listens to the store's broadcast and
//! translates each activity of this program's four public types into a [`ResourceChange`]. Capacity,
//! allocations and shares never arrive: they are written without the log, so the store sends no
//! event for them.

use std::sync::Arc;

use fs_store::{Activity, ActivityKind};
use tokio::sync::broadcast::error::RecvError;

use super::consts::CONTEXT;
use super::handle::ResourceRepo;
use crate::traits::{
    EntityKind, ResourceChange, ResourceEntity, ResourceFeed, ResourceListener, Subscription,
};

fn kind_of(object_type: &str) -> Option<EntityKind> {
    Some(match object_type {
        "Resource" => EntityKind::Resource,
        "Reservation" => EntityKind::Reservation,
        "Need" => EntityKind::Need,
        "Offer" => EntityKind::Offer,
        _ => return None,
    })
}

/// The change an activity stands for, if it is about one of the four public types.
fn change_of(a: &Activity) -> Option<ResourceChange> {
    let set = a.change_set().ok()?;
    let image = set.after.as_ref().or(set.before.as_ref())?;
    if image.envelope.context.as_deref() != Some(CONTEXT) {
        return None;
    }
    let entity = ResourceEntity {
        kind: kind_of(&image.envelope.object_type)?,
        id: a.object,
    };
    match (a.kind, set.after.is_some()) {
        (ActivityKind::Create, _) => Some(ResourceChange::Created(entity)),
        (ActivityKind::Update, _) | (ActivityKind::Undo, true) => {
            Some(ResourceChange::Updated(entity))
        }
        (ActivityKind::Delete | ActivityKind::Undo, _) => Some(ResourceChange::Deleted(entity)),
        _ => None,
    }
}

impl ResourceFeed for ResourceRepo {
    fn subscribe(&self, listener: Arc<dyn ResourceListener>) -> Subscription {
        let mut events = self.events.subscribe();
        Subscription::new(tokio::spawn(async move {
            loop {
                match events.recv().await {
                    Ok(a) => {
                        if let Some(change) = change_of(&a) {
                            listener.changed(&change);
                        }
                    }
                    Err(RecvError::Lagged(_)) => {}
                    Err(RecvError::Closed) => break,
                }
            }
        }))
    }
}
