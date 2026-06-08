//! Py3.12 conformance tests for the `binascii` module (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_binascii.py):
//!   hexlify/unhexlify, b2a_hex/a2b_hex.
//!
//! `binascii.crc32` is intentionally excluded — the JIT path currently
//! raises `AttributeError: 'dict' object has no attribute 'crc32'`
//! while the same call works under `zlib.crc32`. Tracked separately as
//! a deferred runtime gap (CRC32 module-attribute resolution).
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_binascii_hexlify_short() {
    let out = jit_capture(
        r#"import binascii
print(binascii.hexlify(b"hi").decode())
"#,
    );
    assert_output(&out, "6869\n");
}

#[test]
fn test_binascii_unhexlify_short() {
    let out = jit_capture(
        r#"import binascii
print(binascii.unhexlify("6869").decode())
"#,
    );
    assert_output(&out, "hi\n");
}

#[test]
fn test_binascii_hexlify_roundtrip() {
    let out = jit_capture(
        r#"import binascii
data = b"hello world"
encoded = binascii.hexlify(data)
print(binascii.unhexlify(encoded) == data)
"#,
    );
    assert_output(&out, "True\n");
}

#[test]
fn test_binascii_b2a_a2b_hex_alias() {
    let out = jit_capture(
        r#"import binascii
print(binascii.b2a_hex(b"AB").decode())
print(binascii.a2b_hex("4142").decode())
"#,
    );
    assert_output(&out, "4142\nAB\n");
}

