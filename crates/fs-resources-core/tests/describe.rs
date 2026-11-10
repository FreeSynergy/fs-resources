//! The type descriptions against the table, the contract and `requirements.md`.
//!
//! Pattern: none (tests). They go through the public traits (`DescribesType`, `Items::capabilities`),
//! not through the tables' internals, and read a verbatim copy of the requirements page.

mod common;

use std::collections::BTreeSet;

use fs_resources_core::describe::{
    COMMAND_REQUIREMENTS, COMMANDS, FRONTEND_REQUIREMENTS, is_unsupported, registry,
    unsupported_commands,
};
use fs_resources_core::{Allocation, Capacity, Need, Offer, Reservation, Resource};
use fs_types::{BasicType, CommandKind, DescribesType, TypeDescription};

fn all() -> Vec<TypeDescription> {
    vec![
        Resource::describe(),
        Reservation::describe(),
        Need::describe(),
        Offer::describe(),
        Capacity::describe(),
        Allocation::describe(),
    ]
}

/// The first column of the capability table of `requirements.md`.
fn requirement_rows(page: &str) -> BTreeSet<String> {
    page.lines()
        .filter(|l| l.starts_with("| ") && !l.starts_with("| Capability") && !l.starts_with("|--"))
        .filter_map(|l| l.trim_matches('|').split('|').next())
        .map(|c| c.trim().to_owned())
        .collect()
}

#[test]
fn every_description_is_sound_and_the_registry_holds_all_six() {
    let r = registry().expect("the descriptions break a rule of fs-types");
    assert_eq!(r.len(), 6);
    for d in all() {
        d.validate().unwrap();
        assert_eq!(d.program, "fs-resources");
        assert!(!d.fields.is_empty(), "{} has no fields", d.name);
    }
}

#[test]
fn the_description_has_every_command_of_the_table_the_missing_ones_too() {
    for d in all() {
        let named: Vec<&str> = d.commands.iter().map(|c| c.name.as_str()).collect();
        let table: Vec<&str> = COMMANDS
            .iter()
            .filter(|c| c.of == d.name)
            .map(|c| c.name)
            .collect();
        assert_eq!(named, table, "{}", d.name);
    }
    let total: usize = all().iter().map(|d| d.commands.len()).sum();
    assert_eq!(total, COMMANDS.len());
    assert!(unsupported_commands().count() >= 7);
}

#[test]
fn every_requirement_row_is_in_the_table_and_missing_ones_are_unsupported() {
    let rows = requirement_rows(include_str!("fixtures/requirements.md"));
    let known: BTreeSet<String> = FRONTEND_REQUIREMENTS
        .iter()
        .chain(COMMAND_REQUIREMENTS)
        .map(|s| (*s).to_owned())
        .collect();
    assert_eq!(rows, known, "requirements.md and commands.rs disagree");
    for req in COMMAND_REQUIREMENTS {
        let stands: Vec<_> = COMMANDS.iter().filter(|c| c.capability == *req).collect();
        assert!(!stands.is_empty(), "no command for: {req}");
        assert!(
            stands.iter().all(|c| !c.supported),
            "{req} must be marked missing"
        );
    }
    // a command that stands for a requirement is never marked supported by mistake
    assert!(
        COMMANDS
            .iter()
            .filter(|c| COMMAND_REQUIREMENTS.contains(&c.capability))
            .all(|c| !c.supported)
    );
}

#[test]
fn the_core_reports_every_capability_a_supported_command_stands_for() {
    // contract: what the provider says it can do is what the table promises (no group port here)
    let repo = futures_lite_block(async { common::setup().await.0 });
    let report = repo.items().capabilities();
    for c in COMMANDS.iter().filter(|c| c.supported) {
        if c.capability == "capacity.group" {
            assert!(!report.is_supported(c.capability));
        } else {
            assert!(
                report.is_supported(c.capability),
                "{} ({})",
                c.name,
                c.capability
            );
        }
    }
}

#[test]
fn every_type_has_find_for_the_list_and_record_commands_name_the_record() {
    for d in all() {
        let find = d.commands.iter().find(|c| c.kind == CommandKind::Find);
        assert!(find.is_some(), "{} has no find", d.name);
        for c in d
            .commands
            .iter()
            .filter(|c| matches!(c.kind, CommandKind::Change | CommandKind::Delete))
        {
            let id = c.input.iter().find(|f| f.name == "id");
            let id = id.unwrap_or_else(|| panic!("{}.{} has no id", d.name, c.name));
            assert_eq!(id.basic, BasicType::Reference);
            assert_eq!(id.target, d.name);
            assert!(id.required);
        }
    }
}

#[test]
fn change_commands_have_no_required_field_but_the_id() {
    for d in all() {
        for c in d.commands.iter().filter(|c| c.kind == CommandKind::Change) {
            assert!(
                c.input.iter().all(|f| f.name == "id" || !f.required)
                    || is_unsupported(&d.name, &c.name),
                "{}.{} would force fields on a partial change",
                d.name,
                c.name
            );
        }
    }
}

fn futures_lite_block<T>(f: impl std::future::Future<Output = T>) -> T {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(f)
}
