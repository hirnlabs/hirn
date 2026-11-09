//! Watcher debounce and write-echo suppression.

use std::io::Write;
use std::sync::mpsc;
use std::time::Duration;

use hirn_data::tier2::watch::{EchoGuard, FileEventKind};
use hirn_data::{FileEvent, Watcher};

fn write_file(path: &std::path::Path, contents: &str) {
    let mut file = std::fs::File::create(path).unwrap();
    file.write_all(contents.as_bytes()).unwrap();
}

#[test]
fn external_markdown_edits_are_reported() {
    let dir = tempfile::tempdir().unwrap();
    let (sender, receiver) = mpsc::channel();
    let watcher = Watcher::start(dir.path(), move |event: FileEvent| {
        let _ = sender.send(event);
    })
    .unwrap();

    // Give the platform watcher a moment to attach before the first write.
    std::thread::sleep(Duration::from_millis(300));
    write_file(&dir.path().join("note.md"), "# hello\n");

    let event = receiver
        .recv_timeout(Duration::from_secs(10))
        .expect("a change to a Markdown file is reported");
    assert_eq!(event.path.as_str(), "note.md");
    assert!(matches!(
        event.kind,
        FileEventKind::Created | FileEventKind::Modified
    ));
    drop(watcher);
}

#[test]
fn non_markdown_and_hidden_paths_are_ignored() {
    let dir = tempfile::tempdir().unwrap();
    let (sender, receiver) = mpsc::channel();
    let watcher = Watcher::start(dir.path(), move |event: FileEvent| {
        let _ = sender.send(event);
    })
    .unwrap();
    std::thread::sleep(Duration::from_millis(300));

    write_file(&dir.path().join("notes.json"), "{}\n");
    std::fs::create_dir_all(dir.path().join(".hirn")).unwrap();
    write_file(&dir.path().join(".hirn").join("cache.md"), "# hidden\n");

    assert!(
        receiver.recv_timeout(Duration::from_millis(700)).is_err(),
        "neither JSON nor .hirn/ content is surfaced"
    );
    drop(watcher);
}

#[test]
fn rapid_edits_are_coalesced_into_one_event() {
    let dir = tempfile::tempdir().unwrap();
    let (sender, receiver) = mpsc::channel();
    let watcher = Watcher::start(dir.path(), move |event: FileEvent| {
        let _ = sender.send(event);
    })
    .unwrap();
    std::thread::sleep(Duration::from_millis(300));

    // Five saves inside one debounce window must arrive as one event.
    for revision in 0..5 {
        write_file(
            &dir.path().join("burst.md"),
            &format!("# revision {revision}\n"),
        );
        std::thread::sleep(Duration::from_millis(10));
    }

    let first = receiver
        .recv_timeout(Duration::from_secs(10))
        .expect("a debounced event is reported");
    assert_eq!(first.path.as_str(), "burst.md");

    let mut count = 1;
    while let Ok(event) = receiver.recv_timeout(Duration::from_millis(600)) {
        assert_eq!(event.path.as_str(), "burst.md");
        count += 1;
    }
    assert_eq!(count, 1, "a burst of writes is one debounced event");
    drop(watcher);
}

#[test]
fn echo_guard_recognises_our_own_write_once() {
    let guard = EchoGuard::default();
    let path = std::path::Path::new("notes/note.md");
    guard.expect(path, "written by us");

    assert!(
        guard.consume_echo(path, "written by us"),
        "the backend's own write is suppressed"
    );
    assert!(
        !guard.consume_echo(path, "written by us"),
        "suppression is one-shot, so a later identical edit still counts"
    );
    assert!(
        !guard.consume_echo(path, "edited elsewhere"),
        "a different payload is an external edit"
    );
}
