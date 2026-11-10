//! The plain data a description is made of: one field, one command.
//!
//! Pattern: Value object with const builders — the tables in `fields.rs` and `commands.rs` are
//! `const` data, so a reviewer reads them as a list, and `build.rs` turns them into the types of
//! `fs-types` (Adapter). Nothing here knows the wire.

use fs_types::{BasicType, CommandKind};

/// One field of a data type, or one input or result of a command.
#[derive(Clone, Copy, Debug)]
pub struct FieldSpec {
    /// Machine name, lower camel case.
    pub name: &'static str,
    /// The basic type.
    pub basic: BasicType,
    /// Must be given.
    pub required: bool,
    /// A list of values.
    pub multiple: bool,
    /// For a reference: the data type referred to; empty = any.
    pub target: &'static str,
    /// For a choice: the allowed keys.
    pub choices: &'static [&'static str],
}

impl FieldSpec {
    /// A plain field of `basic`.
    pub const fn new(name: &'static str, basic: BasicType) -> Self {
        Self {
            name,
            basic,
            required: false,
            multiple: false,
            target: "",
            choices: &[],
        }
    }

    /// Must be given.
    #[must_use]
    pub const fn req(mut self) -> Self {
        self.required = true;
        self
    }

    /// A list of values.
    #[must_use]
    pub const fn many(mut self) -> Self {
        self.multiple = true;
        self
    }

    /// A reference to `target`.
    #[must_use]
    pub const fn to(mut self, target: &'static str) -> Self {
        self.target = target;
        self
    }
}

/// A text field. Quantities and minutes are text here: the basic types have no number yet (see `DESIGN.md`).
pub const fn text(name: &'static str) -> FieldSpec {
    FieldSpec::new(name, BasicType::Text)
}

/// A date and time with offset.
pub const fn when(name: &'static str) -> FieldSpec {
    FieldSpec::new(name, BasicType::DateTime)
}

/// A reference to an object of `target` (empty = any).
pub const fn reference(name: &'static str, target: &'static str) -> FieldSpec {
    FieldSpec::new(name, BasicType::Reference).to(target)
}

/// One of `choices`.
pub const fn choice(name: &'static str, choices: &'static [&'static str]) -> FieldSpec {
    let mut f = FieldSpec::new(name, BasicType::Choice);
    f.choices = choices;
    f
}

/// One command of a data type, with whether the core can carry it out.
#[derive(Clone, Copy, Debug)]
pub struct CommandSpec {
    /// The data type it belongs to.
    pub of: &'static str,
    /// Machine name, unique per type.
    pub name: &'static str,
    /// What it does to the data.
    pub kind: CommandKind,
    /// Whether the core can do it. `false` = a requirement the core does not meet yet.
    pub supported: bool,
    /// The capability id of the contract that carries it out (`resource.create`), or the requirement
    /// it stands for when `supported` is false.
    pub capability: &'static str,
    /// Its inputs.
    pub input: &'static [FieldSpec],
    /// Its results.
    pub result: &'static [FieldSpec],
}

impl CommandSpec {
    /// A command the core carries out.
    pub const fn done(
        of: &'static str,
        name: &'static str,
        kind: CommandKind,
        capability: &'static str,
        input: &'static [FieldSpec],
        result: &'static [FieldSpec],
    ) -> Self {
        Self {
            of,
            name,
            kind,
            supported: true,
            capability,
            input,
            result,
        }
    }

    /// A command a requirement asks for and the core does not have.
    pub const fn missing(
        of: &'static str,
        name: &'static str,
        kind: CommandKind,
        requirement: &'static str,
        input: &'static [FieldSpec],
        result: &'static [FieldSpec],
    ) -> Self {
        Self {
            of,
            name,
            kind,
            supported: false,
            capability: requirement,
            input,
            result,
        }
    }
}
