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
fn every_type_has_find_and_record_commands_work_on_a_record() {
    for d in all() {
        assert!(
            d.commands.iter().any(|c| c.kind == CommandKind::Find),
            "{} has no find",
            d.name
        );
        let on_record = d.record_commands();
        for c in &d.commands {
            let names_itself = c
                .input
                .iter()
                .any(|f| f.basic == BasicType::Reference && f.target == d.name);
            let is_record = on_record.iter().any(|r| r.name == c.name);
            assert_eq!(
                is_record,
                names_itself || matches!(c.kind, CommandKind::Change | CommandKind::Delete),
                "{}.{}",
                d.name,
                c.name
            );
            if c.kind == CommandKind::Delete {
                let id = c
                    .input
                    .iter()
                    .find(|f| f.name == "id")
                    .expect("delete names the record");
                assert!(id.required && id.target == d.name);
            }
        }
    }
}

#[test]
fn change_is_an_edit_and_takes_no_input_of_its_own() {
    for d in all() {
        for c in d.commands.iter().filter(|c| c.kind == CommandKind::Change) {
            assert!(c.input.is_empty(), "{}.{}", d.name, c.name);
            assert!(c.supported, "{}.{}", d.name, c.name);
        }
    }
}

#[test]
fn the_wire_flag_is_the_table_flag_for_every_command() {
    for d in all() {
        for c in &d.commands {
            let row = COMMANDS
                .iter()
                .find(|r| r.of == d.name && r.name == c.name)
                .unwrap();
            assert_eq!(c.supported, row.supported, "{}.{}", d.name, c.name);
            assert_eq!(c.supported, !is_unsupported(&d.name, &c.name));
        }
    }
    assert!(
        !Resource::describe()
            .find_command("setClass")
            .unwrap()
            .supported
    );
}

fn futures_lite_block<T>(f: impl std::future::Future<Output = T>) -> T {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(f)
}
