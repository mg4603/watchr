use super::*;

#[test]
#[traced_test]
fn test_has_matching_event_no_match() {
    let (file_1, _tmp_dir) = create_temp_file("file_access.txt", "test");
    let (file_2, _tmp_dir) = create_temp_file("readme.md", "test");

    let events = vec![
        create_debounced_event("access", &[file_1.to_str().unwrap()]),
        create_debounced_event("modify", &[file_2.to_str().unwrap()]),
    ];

    let exts = vec!["rs".to_string()];
    assert!(!has_matching_event(&events, Some(&exts)));
    assert!(logs_contain("file_access"));
    assert!(logs_contain("readme.md"));
}

#[test]
#[traced_test]
fn test_has_matching_event_with_short_circuit() {
    let (file_1, _tmp_dir) = create_temp_file("file_access.txt", "test");
    let (file_2, _tmp_dir) = create_temp_file("file_a.rs", "test");
    let (file_3, _tmp_dir) = create_temp_file("file_b.rs", "test");

    let events = vec![
        create_debounced_event("access", &[file_1.to_str().unwrap()]),
        create_debounced_event("create", &[file_2.to_str().unwrap()]),
        create_debounced_event("remove", &[file_3.to_str().unwrap()]),
    ];
    let exts = vec!["rs".to_string()];
    assert!(has_matching_event(&events, Some(&exts)));

    assert!(logs_contain("file_access"));
    assert!(logs_contain("file_a"));
    assert!(!logs_contain("file_b"));
}
