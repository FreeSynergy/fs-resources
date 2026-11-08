//! `supportReport` in Rust.
#![allow(clippy::assert_is_empty)] // tests: plain `is_empty()` asserts read clearer than typed empty comparisons

mod common;

use common::setup;

#[tokio::test]
async fn the_capabilities_are_the_support_report() {
    // `supportReport` has no method of its own: `Capabilities` is its Rust form (entries with
    // `supported` and a reason), and `Capabilities::write` fills the wire report.
    let (repo, _) = setup().await;
    let report = repo.items().capabilities();
    assert_eq!(report.provider(), "fs-resources-core");
    let no = report.entries().iter().find(|e| !e.supported).unwrap();
    assert_eq!(no.capability, "capacity.group");
    assert!(!no.reason.is_empty());
}
