use super::*;

#[test]
fn logs_at_debug_level() {
    let recorder = LevelRecorder(Arc::new(Mutex::new(Vec::new())));
    let _guard = tracing::subscriber::set_default(
        tracing_subscriber::registry()
            .with(tracing_subscriber::filter::LevelFilter::INFO)
            .with(recorder.clone()),
    );

    let (file, _tmp) = create_temp_file("file.txt", "test");
    let event = create_debounced_event("access", vec![file.clone()]);

    log_event(&event);

    // At INFO, a debug-level event must not be recorded
    assert!(recorder.0.lock().unwrap().is_empty());
}

#[test]
#[traced_test]
fn logs_a_single_modified_file() {
    let (file, _tmp) = create_temp_file("file.txt", "test");
    let event = create_debounced_event("modify", vec![file.clone()]);

    log_event(&event);

    assert!(logs_contain("event detail"));
    assert!(logs_contain(&file.display().to_string()));
    assert!(logs_contain("Modify(Any)"));
}

#[test]
#[traced_test]
fn logs_multiple_paths() {
    let (file_1, _tmp) = create_temp_file("a.txt", "test");
    let (file_2, _tmp) = create_temp_file("b.txt", "test");
    let event = create_debounced_event(
        "create",
        vec![file_1.clone(), file_2.clone()],
    );

    log_event(&event);

    assert!(logs_contain(&file_1.display().to_string()));
    assert!(logs_contain(&file_2.display().to_string()));
    assert!(logs_contain("Create(Any)"));
}

#[test]
#[traced_test]
fn logs_removed_file() {
    let (file, _tmp) = create_temp_file("gone.txt", "test");
    let event = create_debounced_event("remove", vec![file.clone()]);
    log_event(&event);

    assert!(logs_contain("event detail"));
    assert!(logs_contain(&file.display().to_string()));
    assert!(logs_contain("Remove"));
}

#[test]
#[traced_test]
fn logs_empty_path_list() {
    let event = create_debounced_event("access", vec![]);
    log_event(&event);

    assert!(logs_contain("event detail"));
    assert!(logs_contain("[]"));
}

#[test]
#[traced_test]
fn logs_path_with_special_chars() {
    let (file, _tmp) = create_temp_file("my_file (1).txt", "test");
    let event = create_debounced_event("modify", vec![file.clone()]);
    log_event(&event);

    assert!(logs_contain("event detail"));
    assert!(logs_contain(&file.display().to_string()));
    assert!(logs_contain("Modify(Any)"));
}

#[test]
#[traced_test]
fn logs_unicode_path() {
    let (file, _tmp) = create_temp_file("héllo_wörld.txt", "test");
    let event = create_debounced_event("create", vec![file.clone()]);
    log_event(&event);

    assert!(logs_contain("event detail"));
    assert!(logs_contain(&file.display().to_string()));
}
