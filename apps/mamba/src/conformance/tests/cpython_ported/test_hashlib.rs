//! Py3.12 conformance tests for the `hashlib` module (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_hashlib.py):
//!   md5, sha1, sha256, sha512 digest values for known inputs.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_hashlib_md5_hello() {
    let out = jit_capture(
        r#"import hashlib
print(hashlib.md5(b"hello").hexdigest())
"#,
    );
    assert_output(&out, "5d41402abc4b2a76b9719d911017c592\n");
}

#[test]
fn test_hashlib_sha1_hello() {
    let out = jit_capture(
        r#"import hashlib
print(hashlib.sha1(b"hello").hexdigest())
"#,
    );
    assert_output(&out, "aaf4c61ddcc5e8a2dabede0f3b482cd9aea9434d\n");
}

#[test]
fn test_hashlib_sha256_hello() {
    let out = jit_capture(
        r#"import hashlib
print(hashlib.sha256(b"hello").hexdigest())
"#,
    );
    assert_output(
        &out,
        "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824\n",
    );
}

#[test]
fn test_hashlib_md5_empty() {
    let out = jit_capture(
        r#"import hashlib
print(hashlib.md5(b"").hexdigest())
"#,
    );
    assert_output(&out, "d41d8cd98f00b204e9800998ecf8427e\n");
}

#[test]
fn test_hashlib_sha256_empty() {
    let out = jit_capture(
        r#"import hashlib
print(hashlib.sha256(b"").hexdigest())
"#,
    );
    assert_output(
        &out,
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855\n",
    );
}

#[test]
fn test_hashlib_sha512_hello() {
    let out = jit_capture(
        r#"import hashlib
print(hashlib.sha512(b"hello").hexdigest())
"#,
    );
    assert_output(
        &out,
        "9b71d224bd62f3785d96d46ad3ea3d73319bfbc2890caadae2dff72519673ca72323c3d99ba5c11d7c7acc6e14b8c5da0c4663475c2e5c3adef46f73bcdec043\n",
    );
}
