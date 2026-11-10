//! From the tables to the descriptions `fs-types` reports, and the `DescribesType` of each type.
//!
//! Pattern: Adapter (table to wire description) + Registry entry — each stored type implements
//! `DescribesType`, so `TypeRegistry::register::<Resource>()` puts it in the catalog the program
//! serves. The labels are translation keys made from the names: `resource.quantity`,
//! `resource.create`, `resource.create.name`.

use fs_types::{
    CommandDescription, DescribesType, FieldDescription, TypeDescription, TypeRegistry,
};

use super::commands::commands_of;
use super::fields::{
    ALLOCATION, ALLOCATION_FIELDS, CAPACITY, CAPACITY_FIELDS, NEED, NEED_FIELDS, OFFER,
    OFFER_FIELDS, RESERVATION, RESERVATION_FIELDS, RESOURCE, RESOURCE_FIELDS,
};
use super::spec::{CommandSpec, FieldSpec};
use crate::model::{Allocation, Capacity, Need, Offer, Reservation, Resource};

/// The program that owns the types (ADR 0050).
pub const PROGRAM: &str = "fs-resources";

/// The standard every type names: none exists for a resource model (DESIGN.md).
const STANDARD: &str = "own model: resources.capnp (no standard found)";

/// `minutesPerWeek` to `minutes-per-week`: the part of a translation key made from a property name.
fn kebab(name: &str) -> String {
    let mut out = String::new();
    for c in name.chars() {
        if c.is_ascii_uppercase() {
            out.push('-');
            out.push(c.to_ascii_lowercase());
        } else {
            out.push(c);
        }
    }
    out
}

fn field(spec: &FieldSpec, label: &str) -> FieldDescription {
    let mut d = FieldDescription::new(spec.name, spec.basic).label(label);
    if spec.required {
        d = d.required();
    }
    if spec.multiple {
        d = d.multiple();
    }
    if !spec.target.is_empty() {
        d = d.target(spec.target);
    }
    if !spec.choices.is_empty() {
        d = d.choices(spec.choices);
    }
    d
}

fn command(of: &str, spec: &CommandSpec) -> CommandDescription {
    let base = format!("{}.{}", of.to_ascii_lowercase(), kebab(spec.name));
    // SWITCH: set .supported(flag) when fs-types has it. Until then the flag stays in the table
    // (`spec.supported`), and a backend refuses the commands that are `false`.
    let mut c = CommandDescription::new(spec.name, spec.kind, &base);
    for f in spec.input {
        c = c.input(field(f, &format!("{base}.{}", kebab(f.name))));
    }
    for f in spec.result {
        c = c.result(field(f, &format!("{base}.{}", kebab(f.name))));
    }
    c
}

/// The description of `name` from the field table and the command table.
fn describe(name: &str, fields: &[FieldSpec]) -> TypeDescription {
    let key = name.to_ascii_lowercase();
    let mut t = TypeDescription::new(name, PROGRAM, STANDARD, &key);
    for f in fields {
        t = t.field(field(f, &format!("{key}.{}", kebab(f.name))));
    }
    for c in commands_of(name) {
        t = t.command(command(name, c));
    }
    t
}

macro_rules! describes {
    ($($ty:ty => $name:expr, $fields:expr;)+) => {$(
        impl DescribesType for $ty {
            fn describe() -> TypeDescription {
                describe($name, $fields)
            }
        }
    )+};
}

describes! {
    Resource => RESOURCE, RESOURCE_FIELDS;
    Reservation => RESERVATION, RESERVATION_FIELDS;
    Need => NEED, NEED_FIELDS;
    Offer => OFFER, OFFER_FIELDS;
    Capacity => CAPACITY, CAPACITY_FIELDS;
    Allocation => ALLOCATION, ALLOCATION_FIELDS;
}

/// The catalog of this program: every type it owns.
///
/// # Errors
/// A description that breaks the rules of `fs-types` (a defect of the tables; a test keeps it out).
pub fn registry() -> Result<TypeRegistry, fs_types::DescriptionProblem> {
    let mut r = TypeRegistry::new();
    r.register::<Resource>()?
        .register::<Reservation>()?
        .register::<Need>()?
        .register::<Offer>()?
        .register::<Capacity>()?
        .register::<Allocation>()?;
    Ok(r)
}
