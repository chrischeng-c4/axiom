//! Py3.12 conformance tests for the `pickle` module (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_pickle.py):
//!   dumps/loads round-trips for primitives, containers, and nested
//!   structures. Object identity (e.g., None) and equality are checked
//!   rather than wire format — pickle wire format is implementation
//!   detail.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_pickle_roundtrip_int() {
    let out = jit_capture(
        r#"import pickle
print(pickle.loads(pickle.dumps(42)) == 42)
"#,
    );
    assert_output(&out, "True\n");
}

#[test]
fn test_pickle_roundtrip_string() {
    let out = jit_capture(
        r#"import pickle
print(pickle.loads(pickle.dumps("hello world")) == "hello world")
"#,
    );
    assert_output(&out, "True\n");
}

#[test]
fn test_pickle_roundtrip_list() {
    let out = jit_capture(
        r#"import pickle
print(pickle.loads(pickle.dumps([1, 2, 3])) == [1, 2, 3])
"#,
    );
    assert_output(&out, "True\n");
}

#[test]
fn test_pickle_roundtrip_tuple() {
    let out = jit_capture(
        r#"import pickle
print(pickle.loads(pickle.dumps((1, 2, 3))) == (1, 2, 3))
"#,
    );
    assert_output(&out, "True\n");
}

#[test]
fn test_pickle_roundtrip_nested_dict() {
    let out = jit_capture(
        r#"import pickle
data = {"a": 1, "b": [2, 3], "c": {"d": 4}}
r = pickle.loads(pickle.dumps(data))
print(r["a"])
print(r["b"])
print(r["c"]["d"])
"#,
    );
    assert_output(&out, "1\n[2, 3]\n4\n");
}

#[test]
fn test_pickle_roundtrip_bool() {
    let out = jit_capture(
        r#"import pickle
print(pickle.loads(pickle.dumps(True)) == True)
print(pickle.loads(pickle.dumps(False)) == False)
"#,
    );
    assert_output(&out, "True\nTrue\n");
}

#[test]
fn test_pickle_roundtrip_none() {
    let out = jit_capture(
        r#"import pickle
print(pickle.loads(pickle.dumps(None)) is None)
"#,
    );
    assert_output(&out, "True\n");
}

#[test]
fn test_pickle_roundtrip_float() {
    let out = jit_capture(
        r#"import pickle
print(pickle.loads(pickle.dumps(3.14)) == 3.14)
"#,
    );
    assert_output(&out, "True\n");
}
