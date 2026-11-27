use super::*;

#[traced_test]
#[test]
fn record_one_error_per_item() {
    log_errors(vec![
        std::io::Error::other("foo"),
        std::io::Error::other("bar"),
    ]);

    assert!(logs_contain("failed to process file watch event"));
    assert!(logs_contain("foo"));
    assert!(logs_contain("bar"));
}

#[traced_test]
#[test]
fn empty_iterator_logs_nothing() {
    let empty: Vec<std::io::Error> = Vec::new();
    log_errors(empty);
    assert!(!logs_contain("failed to process file watch event"));
}

#[traced_test]
#[test]
fn any_display_type_accepted() {
    struct CustomError(String);
    impl std::fmt::Display for CustomError {
        fn fmt(
            &self,
            f: &mut std::fmt::Formatter<'_>,
        ) -> std::fmt::Result {
            write!(f, "custom: {}", self.0)
        }
    }

    log_errors(vec![CustomError("bad path".into())]);
    assert!(logs_contain("failed to process file watch event"));
    assert!(logs_contain("custom: bad path"));
}

#[traced_test]
#[test]
fn works_with_iterators_not_just_vecs() {
    log_errors((0..3).map(|i| std::io::Error::other(format!("e{i}"))));
    assert!(logs_contain("failed to process file watch event"));
    assert!(logs_contain("e0"));
    assert!(logs_contain("e2"));
}
