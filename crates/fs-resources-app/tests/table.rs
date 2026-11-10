//! The table of commands against the backend: unsupported ones are refused, supported ones have a handler.
//!
//! Pattern: none (tests).

mod common;

use common::World;
use fs_admin_frontend::{Backend, BackendError, Values};
use fs_resources_core::describe::{COMMANDS, RESOURCE, unsupported_commands};
use fs_types::CommandKind;

#[test]
fn commands_the_core_does_not_have_are_refused_by_name_and_reported_unsupported() {
    let w = World::new();
    let missing: Vec<_> = unsupported_commands().collect();
    assert!(missing.len() >= 7);
    for c in missing {
        let b = w.of(c.of);
        assert!(!b.supported(c.name), "{}.{}", c.of, c.name);
        let refusal = b
            .run(
                c.name,
                Some("https://example.org/resources/x"),
                &Values::new(),
            )
            .unwrap_err();
        match refusal {
            BackendError::Rejected(issues) => {
                assert_eq!(issues.len(), 1);
                assert_eq!(issues[0].field, c.name, "the finding names the command");
            }
            other => panic!("{}.{}: {other:?}", c.of, c.name),
        }
    }
}

#[test]
fn every_supported_command_of_the_table_has_a_handler() {
    // an empty call may be refused for its input, never answered "no such command"
    let w = World::new();
    for c in COMMANDS.iter().filter(|c| c.supported) {
        let b = w.of(c.of);
        assert!(b.supported(c.name));
        if c.kind == CommandKind::Find {
            b.list(&Values::new())
                .unwrap_or_else(|e| panic!("{}.{}: {e:?}", c.of, c.name));
        } else {
            assert_ne!(
                b.run(c.name, None, &Values::new()),
                Err(BackendError::NotFound),
                "{}.{}",
                c.of,
                c.name
            );
        }
    }
    assert_eq!(
        w.of(RESOURCE).run("nonsense", None, &Values::new()),
        Err(BackendError::NotFound)
    );
}
