use super::*;

#[test]
#[traced_test]
fn short_circuit() {
    let (file_1, _tmp_dir) = create_temp_file("file_access.txt", "test");
    let (file_2, _tmp_dir) = create_temp_file("file_a.rs", "test");
    let (file_3, _tmp_dir) = create_temp_file("file_b.rs", "test");

    let events = vec![
        create_debounced_event("access", vec![file_1]),
        create_debounced_event("create", vec![file_2]),
        create_debounced_event("remove", vec![file_3]),
    ];
    let exts = vec!["rs".to_string()];
    assert!(has_matching_event(&events, Some(&exts)));

    assert!(logs_contain("file_access"));
    assert!(logs_contain("file_a"));
    assert!(!logs_contain("file_b"));
}

#[test]
#[traced_test]
fn matches_when_any_path_relevant() {
    let (file_1, _tmp_dir) = create_temp_file("notes.txt", "test");
    let (file_2, _tmp_dir) = create_temp_file("main.rs", "fn main(){}");

    let events =
        vec![create_debounced_event("modify", vec![file_1, file_2])];
    let exts = vec!["rs".to_string()];
    assert!(has_matching_event(&events, Some(&exts)));
    assert!(logs_contain("notes.txt"));
    assert!(logs_contain("main.rs"));
}

#[test]
#[traced_test]
fn mutation_but_filtered_extension() {
    let (file, _tmp_dir) = create_temp_file("notes.txt", "test");
    let events = vec![create_debounced_event("modify", vec![file])];
    let exts = vec!["rs".to_string()];
    assert!(!has_matching_event(&events, Some(&exts)));
    assert!(logs_contain("notes.txt"));
}

#[test]
#[traced_test]
fn not_mutation_matching_extension() {
    let (file, _tmp_dir) = create_temp_file("notes.txt", "test");
    let events = vec![create_debounced_event("access", vec![file])];
    let exts = vec!["txt".to_string()];
    assert!(!has_matching_event(&events, Some(&exts)));
    assert!(logs_contain("notes.txt"));
}

#[test]
#[traced_test]
fn mixed_events_no_match() {
    let (file_1, _tmp_dir) = create_temp_file("main.rs", "fn main(){}");
    let (file_2, _tmp_dir) = create_temp_file("notes.txt", "test");

    let events = vec![
        create_debounced_event("access", vec![file_1]),
        create_debounced_event("modify", vec![file_2]),
    ];

    let exts = vec!["rs".to_string()];
    assert!(!has_matching_event(&events, Some(&exts)));
    assert!(logs_contain("main.rs"));
    assert!(logs_contain("notes.txt"));
}

#[test]
#[traced_test]
fn empty_events_no_match() {
    let exts = vec!["txt".to_string()];
    assert!(!has_matching_event(&[], Some(&exts)));
    assert!(!logs_contain("event details"));
}
