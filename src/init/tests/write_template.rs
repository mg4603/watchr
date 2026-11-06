use super::*;
#[test]
fn happy_path() {
    let mut buf: Vec<u8> = Vec::new();
    write_template(&mut buf, "hello").unwrap();
    assert_eq!(buf, "hello".as_bytes());
}

#[test]
fn write_errors_propagate() {
    let mut writer = FailingWriter;
    let err = write_template(&mut writer, "hello").unwrap_err();
    assert!(matches!(
                err, InitError::Io(e) if e.kind() == std::io::ErrorKind::Other));
}

#[test]
fn flush_errors_propagate() {
    let mut writer = FlushFailingWriter;
    let err = write_template(&mut writer, "hello").unwrap_err();
    assert!(
        matches!(err, InitError::Io(e) if e.kind() == std::io::ErrorKind::Other)
    );
}
