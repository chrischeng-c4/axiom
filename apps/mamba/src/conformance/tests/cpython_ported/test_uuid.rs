//! Py3.12 conformance tests for the `uuid` module (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_uuid.py):
//!   uuid4 string shape (length, dash positions). Property tests only —
//!   uuid4 is random by design.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_uuid_uuid4_string_length() {
    let out = jit_capture(
        r#"import uuid
u = uuid.uuid4()
print(len(str(u)))
"#,
    );
    assert_output(&out, "36\n");
}

#[test]
fn test_uuid_uuid4_dash_count() {
    let out = jit_capture(
        r#"import uuid
u = uuid.uuid4()
print(str(u).count("-"))
"#,
    );
    assert_output(&out, "4\n");
}

#[test]
fn test_uuid_distinct_uuids() {
    let out = jit_capture(
        r#"import uuid
a = uuid.uuid4()
b = uuid.uuid4()
print(a != b)
"#,
    );
    assert_output(&out, "True\n");
}
