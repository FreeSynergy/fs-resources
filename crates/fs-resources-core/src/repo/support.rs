//! What this provider reports it can do.
//!
//! Pattern: Null Object (report) — one place that lists every capability id of the contract; the
//! one that depends on a port says so plainly when the port is missing (ADR 0045).

use fs_interfaces::Capabilities;

use super::handle::ResourceRepo;

/// Capability ids that `ResourceRepo` always provides.
const SUPPORTED: [&str; 17] = [
    "resource.create",
    "resource.edit",
    "resource.delete",
    "resource.read",
    "resource.subscribe",
    "reservation.request",
    "reservation.decide",
    "reservation.read",
    "capacity.set",
    "capacity.read",
    "capacity.share",
    "allocation.propose",
    "allocation.answer",
    "need.create",
    "need.read",
    "offer.make",
    "offer.decide",
];

impl ResourceRepo {
    pub(super) fn support(&self) -> Capabilities {
        let mut c = Capabilities::new("fs-resources-core");
        for id in SUPPORTED {
            c.supported(id);
        }
        if self.groups.is_some() {
            c.supported("capacity.group");
        } else {
            c.unsupported("capacity.group", "no group lookup is set (with_groups)");
        }
        c
    }
}
