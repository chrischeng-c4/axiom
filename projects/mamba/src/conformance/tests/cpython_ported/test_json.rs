//! Py3.12 conformance tests for the `json` module (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_json/):
//!   json.dumps / json.loads round-trips, basic types, nested structures.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_json_dumps_int() {
    let out = jit_capture(
        r#"import json
print(json.dumps(42))
"#,
    );
    assert_output(&out, "42\n");
}

#[test]
fn test_json_dumps_string() {
    let out = jit_capture(
        r#"import json
print(json.dumps("hello"))
"#,
    );
    assert_output(&out, "\"hello\"\n");
}

#[test]
fn test_json_dumps_list() {
    let out = jit_capture(
        r#"import json
print(json.dumps([1, 2, 3]))
"#,
    );
    assert_output(&out, "[1, 2, 3]\n");
}

#[test]
fn test_json_dumps_dict() {
    let out = jit_capture(
        r#"import json
print(json.dumps({"a": 1, "b": [2, 3]}))
"#,
    );
    assert_output(&out, "{\"a\": 1, \"b\": [2, 3]}\n");
}

#[test]
fn test_json_dumps_true_false_null() {
    let out = jit_capture(
        r#"import json
print(json.dumps(True))
print(json.dumps(False))
print(json.dumps(None))
"#,
    );
    assert_output(&out, "true\nfalse\nnull\n");
}

#[test]
fn test_json_loads_int() {
    let out = jit_capture(
        r#"import json
print(json.loads("42"))
"#,
    );
    assert_output(&out, "42\n");
}

#[test]
fn test_json_loads_dict() {
    let out = jit_capture(
        r#"import json
d = json.loads('{"x": 10, "y": 20}')
print(d["x"])
print(d["y"])
"#,
    );
    assert_output(&out, "10\n20\n");
}

#[test]
fn test_json_loads_list() {
    let out = jit_capture(
        r#"import json
print(json.loads("[1, 2, 3]"))
"#,
    );
    assert_output(&out, "[1, 2, 3]\n");
}

#[test]
fn test_json_loads_true_false_null() {
    let out = jit_capture(
        r#"import json
print(json.loads("true"))
print(json.loads("false"))
print(json.loads("null"))
"#,
    );
    assert_output(&out, "True\nFalse\nNone\n");
}

#[test]
fn test_json_roundtrip_nested() {
    let out = jit_capture(
        r#"import json
original = {"users": [{"id": 1, "name": "a"}, {"id": 2, "name": "b"}]}
encoded = json.dumps(original)
decoded = json.loads(encoded)
print(decoded["users"][0]["name"])
print(decoded["users"][1]["id"])
"#,
    );
    assert_output(&out, "a\n2\n");
}
