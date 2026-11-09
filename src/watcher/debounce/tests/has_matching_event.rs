use super::*;

#[test]
#[traced_test]
fn test_has_matching_event_no_match() {
    let tmp_dir = TempDir::new().unwrap();
    let file_1 = tmp_dir.path().join("file_access.txt");
    fs::write(&file_1, "test").unwrap();
    let file_2 = tmp_dir.path().join("readme.md");
    fs::write(&file_2, "test").unwrap();

    let events = vec![
        create_debounced_event("access", &file_1.display().to_string()),
        create_debounced_event("modify", &file_2.display().to_string()),
    ];

    let exts = vec!["rs".to_string()];
    assert!(!has_matching_event(&events, Some(&exts)));

    assert!(logs_contain("file_access"));
    assert!(logs_contain("readme.md"));
}

#[test]
#[traced_test]
fn test_has_matching_event_with_short_circuit() {
    let tmp_dir = TempDir::new().unwrap();
    let file_1 = tmp_dir.path().join("file_access.txt");
    fs::write(&file_1, "test").unwrap();
    let file_2 = tmp_dir.path().join("file_a.rs");
    fs::write(&file_2, "test").unwrap();
    let file_3 = tmp_dir.path().join("file_b.rs");
    fs::write(&file_3, "test").unwrap();

    let events = vec![
        create_debounced_event("access", &file_1.display().to_string()),
        create_debounced_event("create", &file_2.display().to_string()),
        create_debounced_event("remove", &file_3.display().to_string()),
    ];
    let exts = vec!["rs".to_string()];
    assert!(has_matching_event(&events, Some(&exts)));

    assert!(logs_contain("file_access"));
    assert!(logs_contain("file_a"));
    assert!(!logs_contain("file_b"));
}
