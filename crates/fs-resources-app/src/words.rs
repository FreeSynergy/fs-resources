//! The English words of the program.
//!
//! Pattern: Strategy (`Say` of `fs-components-fields`) — keys in, text out. The frontend's own keys
//! are listed here; the keys of the description (`resource.quantity`, `resource.kind.thing`) get
//! the words of their last step (`quantity` -> "Quantity", `minutes-per-week` -> "Minutes per week").
//! Translation (ADR 0053) replaces this file's table, not its users.

use fs_components_fields::Say;

/// The title of the window.
pub const TITLE: &str = "FreeSynergy Resources";

/// The words the generated frontend asks for by key.
const FIXED: &[(&str, &str)] = &[
    ("action.add", "Add"),
    ("action.back", "Back"),
    ("action.cancel", "Cancel"),
    ("action.close", "Close"),
    ("action.remove", "Remove"),
    ("action.save", "Save"),
    ("menu.program", "Program"),
    ("menu.view", "View"),
    ("menu.help", "Help"),
    ("menu.quit", "Quit"),
    ("menu.about", "About"),
    ("message.done", "Done"),
    ("message.rejected", "The program refused this"),
    ("message.not-found", "Not found"),
    ("message.unavailable", "The program cannot be reached"),
    ("message.unsaved", "There is text that was not sent"),
    ("sort.ascending", "ascending"),
    ("sort.descending", "descending"),
    ("finding.slot", "value {n}"),
    ("list.empty", "Nothing here yet"),
    ("list.separator", ", "),
    ("list.stale", "Old data: the program could not be reached"),
    ("detail.not-loaded", "This record is not loaded"),
    ("confirm.sure", "Are you sure?"),
    ("command.soon", "Coming soon"),
    ("value.none", "-"),
    ("about.version", "Version"),
    ("about.origin", "Origin"),
    ("about.licence", "Licence"),
    ("about.repository", "Repository"),
    ("about.unoffered", "Commands not offered here"),
    ("problem.missing", "Needed"),
    ("problem.unknown-field", "Not known to this program"),
    ("problem.not-multiple", "Only one value"),
    ("problem.empty", "Empty"),
    ("problem.too-long", "Too long"),
    ("problem.character", "Not a valid value"),
    (
        "problem.date-time.bad",
        "Not a date and time like 2026-03-02T09:00:00Z",
    ),
    ("problem.date.bad", "Not a date like 2026-03-02"),
    ("problem.uri.not-absolute", "Not a full address (IRI)"),
    ("problem.choice.not-in-list", "Not one of the choices"),
];

/// The words of the last step of a key: `resource.minutes-per-week` -> "Minutes per week".
fn from_last_step(key: &str) -> String {
    let step = key.rsplit('.').next().unwrap_or(key).replace('-', " ");
    let mut chars = step.chars();
    chars
        .next()
        .map(|c| c.to_uppercase().chain(chars).collect())
        .unwrap_or_default()
}

/// The words of this program.
pub struct Words;

impl Say for Words {
    fn say(&self, key: &str) -> String {
        FIXED
            .iter()
            .find(|(k, _)| *k == key)
            .map_or_else(|| from_last_step(key), |(_, text)| (*text).to_owned())
    }
}
