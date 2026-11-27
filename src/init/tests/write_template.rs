use super::*;

// --------------- happy path ------------------
//
#[test]
fn writes_template_bytes_exactly() {
    let template = "key = \"value\"\n";
    let mut buf: Vec<u8> = Vec::new();

    write_template(&mut buf, template).unwrap();
    assert_eq!(buf, template.as_bytes());
}

#[test]
fn works_with_cursor_writer() {
    let template = "template content";
    let mut cursor = Cursor::new(Vec::new());

    write_template(&mut cursor, template).unwrap();
    assert_eq!(cursor.into_inner(), template.as_bytes());
}

#[test]
fn empty_template_writes_nothing() {
    let mut buf: Vec<u8> = Vec::new();

    write_template(&mut buf, "").unwrap();
    assert!(buf.is_empty());
}

#[test]
fn handles_unicode_content() {
    let template = "é = 🎉";
    let mut buf: Vec<u8> = Vec::new();

    write_template(&mut buf, template).unwrap();
    assert_eq!(String::from_utf8(buf).unwrap(), template);
}

#[test]
fn appends_when_writer_already_has_data() {
    let mut buf: Vec<u8> = b"existing\n".to_vec();

    write_template(&mut buf, "new content").unwrap();
    assert_eq!(buf, b"existing\nnew content");
}

// ---------------- Error paths ------------------------

#[test]
fn returns_io_error_when_write_all_fails() {
    let mut writer = FailingWriter { remaining: 4 };
    let err = write_template(&mut writer, "way too long").unwrap_err();
    assert!(matches!(
                err, InitError::Io(e) if e.kind() == std::io::ErrorKind::Other));
}

#[test]
fn returns_io_error_when_flush_fails() {
    let mut writer = FlushFailingWriter;
    let err = write_template(&mut writer, "hello").unwrap_err();

    match err {
        InitError::Io(e) => assert_eq!(e.to_string(), "flush failed"),
        other => panic!("expected Io error, got: {other:?}"),
    }
}

// force buffer overflow to make write_all fail
#[cfg(target_os = "linux")]
#[test]
fn write_template_bufwriter_write_failure() {
    let file = fs::File::create("/dev/full").unwrap();
    let mut writer = BufWriter::with_capacity(1, file);
    let err =
        write_template(&mut writer, DEFAULT_CONFIG_TEMPLATE).unwrap_err();
    assert!(matches!(err, InitError::Io(_)))
}

// template fits in buffer so write_all succeeds and the error surfaces at flush
#[cfg(target_os = "linux")]
#[test]
fn write_template_bufwriter_flush_failure() {
    let file = fs::File::create("/dev/full").unwrap();
    let mut writer = BufWriter::new(file);
    let err =
        write_template(&mut writer, DEFAULT_CONFIG_TEMPLATE).unwrap_err();
    assert!(matches!(err, InitError::Io(_)))
}
