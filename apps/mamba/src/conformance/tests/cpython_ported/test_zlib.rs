//! Py3.12 conformance tests for the `zlib` module (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_zlib.py):
//!   compress/decompress round-trips, crc32, adler32.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_zlib_compress_decompress_roundtrip() {
    let out = jit_capture(
        r#"import zlib
data = b"hello world"
c = zlib.compress(data)
print(zlib.decompress(c) == data)
"#,
    );
    assert_output(&out, "True\n");
}

#[test]
fn test_zlib_crc32_known_value() {
    let out = jit_capture(
        r#"import zlib
print(zlib.crc32(b"hello"))
"#,
    );
    assert_output(&out, "907060870\n");
}

#[test]
fn test_zlib_decompress_handles_repeated() {
    let out = jit_capture(
        r#"import zlib
data = b"abc" * 100
c = zlib.compress(data)
print(zlib.decompress(c) == data)
print(len(c) < len(data))
"#,
    );
    assert_output(&out, "True\nTrue\n");
}
