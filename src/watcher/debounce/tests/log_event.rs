use super::*;

#[test]
fn logs_at_debug_level() {
    let recorder = LevelRecorder(Arc::new(Mutex::new(Vec::new())));
    let _guard = tracing::subscriber::set_default(
        tracing_subscriber::registry()
            .with(tracing_subscriber::filter::LevelFilter::INFO)
            .with(recorder.clone()),
    );

    let event = create_debounced_event("access", &["/tmp/file.txt"]);
    log_event(&event);

    // At INFO, a debug-level event must not be recorded
    assert!(recorder.0.lock().unwrap().is_empty());
}

#[test]
#[traced_test]
fn logs_a_single_modified_file() {
    let event =
        create_debounced_event("modify", &["/tmp/watched/file.txt"]);
    log_event(&event);

    assert!(logs_contain("event detail"));
    assert!(logs_contain("/tmp/watched/file.txt"));
    assert!(logs_contain("Modify(Any)"));
}

#[test]
#[traced_test]
fn logs_multiple_paths() {
    let event = create_debounced_event("create", &["/tmp/a", "/tmp/b"]);
    log_event(&event);

    assert!(logs_contain("/tmp/a"));
    assert!(logs_contain("/tmp/b"));
    assert!(logs_contain("Create(Any)"));
}

#[test]
#[traced_test]
fn logs_removed_file() {
    let event =
        create_debounced_event("remove", &["/tmp/watched/gone.txt"]);
    log_event(&event);

    assert!(logs_contain("event detail"));
    assert!(logs_contain("/tmp/watched/gone.txt"));
    assert!(logs_contain("Remove"));
}

#[test]
#[traced_test]
fn logs_empty_path_list() {
    let event = create_debounced_event("access", &[]);
    log_event(&event);

    assert!(logs_contain("event detail"));
    assert!(logs_contain("[]"));
}

#[test]
#[traced_test]
fn logs_path_with_special_chars() {
    let event =
        create_debounced_event("modify", &["/tmp/my file (1).txt"]);
    log_event(&event);

    assert!(logs_contain("event detail"));
    assert!(logs_contain("/tmp/my file (1).txt"));
    assert!(logs_contain("Modify(Any)"));
}

#[test]
#[traced_test]
fn logs_unicode_path() {
    let event =
        create_debounced_event("create", &["/tmp/héllo_wörld.txt"]);
    log_event(&event);

    assert!(logs_contain("event detail"));
    assert!(logs_contain("/tmp/héllo_wörld.txt"));
}
