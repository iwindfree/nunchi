use ex_00_02_b::describe_size;

#[test]
fn formats_byte_count() {
    assert_eq!(describe_size(0), "0 bytes");
    assert_eq!(describe_size(1024), "1024 bytes");
}
