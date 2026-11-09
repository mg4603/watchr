use super::*;

#[test]
fn test_check_extensions() {
    let filter: Vec<String> = vec!["rs".to_string()];
    assert!(check_extensions(Path::new("src/main.rs"), None));
    assert!(check_extensions(Path::new("src/main.rs"), Some(&filter)));
    assert!(!check_extensions(Path::new("src/main.py"), Some(&filter)));
}

#[test]
fn matching_is_case_sensitive() {
    let filter: Vec<String> = vec!["rs".to_string()];
    assert!(!check_extensions(Path::new("MAIN.RS"), Some(&filter)));
}

#[cfg(unix)]
#[test]
fn non_utf8_extension_is_rejected_not_panicking() {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;
    let path = Path::new(OsStr::from_bytes(b"file.\xff"));
    let filter: Vec<String> = vec!["rs".to_string()];
    assert!(!check_extensions(path, Some(&filter)));
}

#[test]
fn only_the_final_extension_matches() {
    let tar: Vec<String> = vec!["tar".to_string()];
    assert!(!check_extensions(Path::new("archive.tar.gz"), Some(&tar)));

    let gz: Vec<String> = vec!["gz".to_string()];
    assert!(check_extensions(Path::new("archive.tar.gz"), Some(&gz)));
}

#[test]
fn empty_fitler_is_treated_as_no_filter() {
    let filter: Vec<String> = vec![];
    assert!(check_extensions(Path::new("main.rs"), Some(&filter)));
}
