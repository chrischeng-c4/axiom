//! Ported from Lib/test/test_json_ported.py
//! Integration tests: stdlib/json.rs

use super::super::harness::*;

/// Ported from `Lib/test/test_json_ported.py`.
#[test]
fn test_json_dumps_int() {
    let out = jit_capture(
        r#"import json
print(json.dumps(42))
"#,
    );
    assert_output(&out, "42\n");
}

/// Ported from `Lib/test/test_json_ported.py`.
#[test]
fn test_json_dumps_string() {
    let out = jit_capture(
        r#"import json
print(json.dumps("hello"))
"#,
    );
    assert_output(&out, "\"hello\"\n");
}

/// Ported from `Lib/test/test_json_ported.py`.
#[test]
fn test_json_dumps_list() {
    let out = jit_capture(
        r#"import json
print(json.dumps([1, 2, 3]))
"#,
    );
    assert_output(&out, "[1, 2, 3]\n");
}

/// Ported from `Lib/test/test_json_ported.py`.
#[test]
fn test_json_dumps_dict() {
    let out = jit_capture(
        r#"import json
print(json.dumps({"a": 1, "b": [2, 3]}))
"#,
    );
    assert_output(&out, "{\"a\": 1, \"b\": [2, 3]}\n");
}

/// Ported from `Lib/test/test_json_ported.py`.
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

/// Ported from `Lib/test/test_json_ported.py`.
#[test]
fn test_json_loads_int() {
    let out = jit_capture(
        r#"import json
print(json.loads("42"))
"#,
    );
    assert_output(&out, "42\n");
}

/// Ported from `Lib/test/test_json_ported.py`.
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

/// Ported from `Lib/test/test_json_ported.py`.
#[test]
fn test_json_loads_list() {
    let out = jit_capture(
        r#"import json
print(json.loads("[1, 2, 3]"))
"#,
    );
    assert_output(&out, "[1, 2, 3]\n");
}

/// Ported from `Lib/test/test_json_ported.py`.
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

/// Ported from `Lib/test/test_json_ported.py`.
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

