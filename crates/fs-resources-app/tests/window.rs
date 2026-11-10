//! The window: six screens made from the descriptions, over the real core.
//!
//! Pattern: none (tests). Through the shell's public API (`with_shell`, menus), and the words.

use std::sync::Arc;

use fs_components_fields::Say;
use fs_objects::TursoStore;
use fs_resources_app::{Ports, app_widget, words::Words};
use fs_resources_core::ResourceRepo;

fn world() -> (tokio::runtime::Runtime, Ports) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let repo = rt.block_on(async {
        let main = Arc::new(TursoStore::open_in_memory().await.unwrap());
        let private = Arc::new(TursoStore::open_in_memory().await.unwrap());
        ResourceRepo::new(main, private, "https://example.org/people/me")
    });
    (rt, Ports::from_repo(&repo))
}

#[test]
fn the_window_has_the_standard_menus_and_a_screen_per_type() {
    let (rt, ports) = world();
    let app = app_widget(&ports, rt.handle()).unwrap();
    app.with_shell(|s| {
        let titles: Vec<&str> = s.menus().iter().map(|m| m.label.as_str()).collect();
        assert_eq!(titles, ["Program", "View", "Help"]);
        let views: Vec<&str> = s.menus()[1]
            .entries
            .iter()
            .map(|e| e.label.as_str())
            .collect();
        assert_eq!(
            views,
            [
                "Allocation",
                "Capacity",
                "Need",
                "Offer",
                "Reservation",
                "Resource"
            ]
        );
        assert_eq!(s.title(), "FreeSynergy Resources");
    });
}

#[test]
fn words_come_from_the_table_or_from_the_last_step_of_the_key() {
    let w = Words;
    assert_eq!(w.say("command.soon"), "Coming soon");
    assert_eq!(w.say("resource.minutes-per-week"), "Minutes per week");
    assert_eq!(w.say("resource.kind.thing"), "Thing");
    assert_eq!(w.say("capacity.minutes-per-week"), "Minutes per week");
}
