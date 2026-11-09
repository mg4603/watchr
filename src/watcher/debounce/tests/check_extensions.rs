use super::*;

#[test]
fn test_check_extensions() {
    let exts: Vec<String> = vec!["rs".to_string()];
    assert!(check_extensions(Path::new("src/main.rs"), None));
    assert!(check_extensions(Path::new("src/main.rs"), Some(&exts)));
    assert!(!check_extensions(Path::new("src/main.py"), Some(&exts)));
}
