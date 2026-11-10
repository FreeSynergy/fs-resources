//! The command line.
//!
//! Pattern: none (tests).

use fs_resources_app::args::parse;

fn words(w: &[&str]) -> Vec<String> {
    w.iter().map(|s| (*s).to_owned()).collect()
}

#[test]
fn options_are_read_and_unknown_words_are_refused() {
    let a = parse(&words(&[
        "--memory", "--size", "800x600", "--data", "/tmp/x",
    ]))
    .unwrap();
    assert!(a.memory);
    assert_eq!(a.size, (800, 600));
    assert_eq!(a.data.as_deref(), Some(std::path::Path::new("/tmp/x")));
    assert!(parse(&words(&["--size", "0x0"])).is_err());
    assert!(parse(&words(&["--data"])).is_err());
    assert!(
        parse(&words(&["nonsense"]))
            .unwrap_err()
            .contains("nonsense")
    );
    assert!(parse(&words(&["--help"])).unwrap().help);
}
