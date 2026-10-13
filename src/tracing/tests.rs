use super::*;

#[test]
fn test_verbosity_to_level() {
    assert_eq!(verbosity_to_level(0), "off");
    assert_eq!(verbosity_to_level(1), "info");
    assert_eq!(verbosity_to_level(2), "debug");
    assert_eq!(verbosity_to_level(3), "trace");
}
