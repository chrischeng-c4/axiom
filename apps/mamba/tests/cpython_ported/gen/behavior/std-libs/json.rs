use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/json/custom_encoder_default_hook.py`.
#[test]
fn test_gen_behavior_std_libs_json_custom_encoder_default_hook() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "json"
# dimension = "behavior"
# case = "custom_encoder_default_hook"
# subject = "json.JSONEncoder"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/json/test_default.py"
# status = "filled"
# ///
"""json.JSONEncoder: a JSONEncoder subclass overriding default() serializes an otherwise-unsupported type (a set encoded as a sorted list)"""
import json


class SetEncoder(json.JSONEncoder):
    def default(self, obj):
        if isinstance(obj, set):
            return sorted(obj)
        return super().default(obj)


encoded = json.dumps({3, 1, 2}, cls=SetEncoder)
assert json.loads(encoded) == [1, 2, 3], f"custom encoder = {json.loads(encoded)!r}"

print("custom_encoder_default_hook OK")
"###);
    assert_output(&out, r###"custom_encoder_default_hook OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/json/decode_error_attributes.py`.
#[test]
fn test_gen_behavior_std_libs_json_decode_error_attributes() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "json"
# dimension = "behavior"
# case = "decode_error_attributes"
# subject = "json.JSONDecodeError"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/json/test_fail.py"
# status = "filled"
# ///
"""json.JSONDecodeError: a JSONDecodeError raised by loads exposes msg/pos/lineno/colno locating the malformed token"""
import json

raised = None
try:
    json.loads('{"a": 1,\n"b": ?}')
except json.JSONDecodeError as e:
    raised = e
assert raised is not None, "malformed JSON must raise JSONDecodeError"
assert hasattr(raised, "msg"), "JSONDecodeError has msg"
assert isinstance(raised.pos, int), raised.pos
assert raised.lineno == 2, raised.lineno
assert isinstance(raised.colno, int) and raised.colno > 0, raised.colno

print("decode_error_attributes OK")
"###);
    assert_output(&out, r###"decode_error_attributes OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/json/dumps_scalar_type_mapping.py`.
#[test]
fn test_gen_behavior_std_libs_json_dumps_scalar_type_mapping() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "json"
# dimension = "behavior"
# case = "dumps_scalar_type_mapping"
# subject = "json.dumps"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/json/test_pass1.py"
# status = "filled"
# ///
"""json.dumps: Python scalars map to JSON tokens: True->true, False->false, None->null, int->number, float->number, str->quoted"""
import json

assert json.dumps(True) == "true", json.dumps(True)
assert json.dumps(False) == "false", json.dumps(False)
assert json.dumps(None) == "null", json.dumps(None)
assert json.dumps(1) == "1", json.dumps(1)
assert json.dumps(42) == "42", json.dumps(42)
assert json.dumps(1.5) == "1.5", json.dumps(1.5)
assert json.dumps(3.14) == "3.14", json.dumps(3.14)
assert json.dumps("hello") == '"hello"', json.dumps("hello")

print("dumps_scalar_type_mapping OK")
"###);
    assert_output(&out, r###"dumps_scalar_type_mapping OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/json/empty_containers_roundtrip.py`.
#[test]
fn test_gen_behavior_std_libs_json_empty_containers_roundtrip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "json"
# dimension = "behavior"
# case = "empty_containers_roundtrip"
# subject = "json.dumps"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""json.dumps: empty list and empty dict serialize to [] and {} and load back to equal empty containers"""
import json

assert json.dumps([]) == "[]", json.dumps([])
assert json.dumps({}) == "{}", json.dumps({})
assert json.loads("[]") == [], json.loads("[]")
assert json.loads("{}") == {}, json.loads("{}")

print("empty_containers_roundtrip OK")
"###);
    assert_output(&out, r###"empty_containers_roundtrip OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/json/ensure_ascii_escaping.py`.
#[test]
fn test_gen_behavior_std_libs_json_ensure_ascii_escaping() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "json"
# dimension = "behavior"
# case = "ensure_ascii_escaping"
# subject = "json.dumps"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/json/test_unicode.py"
# status = "filled"
# ///
"""json.dumps: non-ASCII characters are \\uXXXX-escaped by default but emitted verbatim when ensure_ascii=False; both round-trip"""
import json

escaped = json.dumps("café")
assert escaped == '"caf\\u00e9"', f"escaped = {escaped!r}"
assert json.loads(escaped) == "café", "escaped unicode round-trip"

raw = json.dumps({"key": "café"}, ensure_ascii=False)
assert "café" in raw, f"unicode verbatim = {raw!r}"
assert json.loads(raw) == {"key": "café"}, "verbatim unicode round-trip"

print("ensure_ascii_escaping OK")
"###);
    assert_output(&out, r###"ensure_ascii_escaping OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/json/float_token_parsing.py`.
#[test]
fn test_gen_behavior_std_libs_json_float_token_parsing() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "json"
# dimension = "behavior"
# case = "float_token_parsing"
# subject = "json.loads"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/json/test_float.py"
# status = "filled"
# ///
"""json.loads: exponent and signed-zero float tokens parse correctly: 1e2 is 100.0 and -0.0 equals 0.0"""
import json

assert json.loads("1e2") == 100.0, f"1e2 = {json.loads('1e2')!r}"
assert isinstance(json.loads("1e2"), float), json.loads("1e2")
assert json.loads("-0.0") == 0.0, f"-0.0 = {json.loads('-0.0')!r}"
assert json.loads("3.14") == 3.14, json.loads("3.14")

print("float_token_parsing OK")
"###);
    assert_output(&out, r###"float_token_parsing OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/json/indent_int_pretty_prints.py`.
#[test]
fn test_gen_behavior_std_libs_json_indent_int_pretty_prints() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "json"
# dimension = "behavior"
# case = "indent_int_pretty_prints"
# subject = "json.dumps"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/json/test_indent.py"
# status = "filled"
# ///
"""json.dumps: integer indent emits a newline plus N spaces per nesting level and never leaves trailing whitespace on a line"""
import json

assert json.dumps([1, 2], indent=2) == "[\n  1,\n  2\n]", repr(json.dumps([1, 2], indent=2))

# When indent is set, the item separator loses its trailing space, so no line
# carries trailing whitespace.
out = json.dumps({"b": 1, "a": 2}, indent=2)
assert out.count("\n") >= 2, repr(out)
assert " \n" not in out, f"trailing space leaked: {out!r}"

print("indent_int_pretty_prints OK")
"###);
    assert_output(&out, r###"indent_int_pretty_prints OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/json/indent_string_tab.py`.
#[test]
fn test_gen_behavior_std_libs_json_indent_string_tab() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "json"
# dimension = "behavior"
# case = "indent_string_tab"
# subject = "json.dumps"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/json/test_indent.py"
# status = "filled"
# ///
"""json.dumps: a string indent of a tab indents each level with a literal tab character (json.tool --tab equivalent)"""
import json

assert json.dumps([1, 2], indent="\t") == "[\n\t1,\n\t2\n]", repr(json.dumps([1, 2], indent="\t"))

print("indent_string_tab OK")
"###);
    assert_output(&out, r###"indent_string_tab OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/json/indent_zero_keeps_newlines.py`.
#[test]
fn test_gen_behavior_std_libs_json_indent_zero_keeps_newlines() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "json"
# dimension = "behavior"
# case = "indent_zero_keeps_newlines"
# subject = "json.dumps"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/json/test_indent.py"
# status = "filled"
# ///
"""json.dumps: indent=0 still emits newlines between items but with no leading spaces"""
import json

assert json.dumps([1, 2], indent=0) == "[\n1,\n2\n]", repr(json.dumps([1, 2], indent=0))

print("indent_zero_keeps_newlines OK")
"###);
    assert_output(&out, r###"indent_zero_keeps_newlines OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/json/large_integer_roundtrip.py`.
#[test]
fn test_gen_behavior_std_libs_json_large_integer_roundtrip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "json"
# dimension = "behavior"
# case = "large_integer_roundtrip"
# subject = "json.dumps"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""json.dumps: a large integer (10**15) survives the dumps->loads round-trip with exact value and int type preserved"""
import json

big = 10 ** 15
rt = json.loads(json.dumps(big))
assert rt == big, f"big int round-trip = {rt!r}"
assert isinstance(rt, int), rt

print("large_integer_roundtrip OK")
"###);
    assert_output(&out, r###"large_integer_roundtrip OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/json/loads_scalar_type_mapping.py`.
#[test]
fn test_gen_behavior_std_libs_json_loads_scalar_type_mapping() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "json"
# dimension = "behavior"
# case = "loads_scalar_type_mapping"
# subject = "json.loads"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/json/test_pass1.py"
# status = "filled"
# ///
"""json.loads: JSON tokens map back to Python types: true is True, false is False, null is None, integer->int, decimal->float"""
import json

assert json.loads("true") is True, "true -> True"
assert json.loads("false") is False, "false -> False"
assert json.loads("null") is None, "null -> None"
assert isinstance(json.loads("1"), int), "integer -> int"
assert json.loads("42") == 42, json.loads("42")
assert isinstance(json.loads("1.5"), float), "decimal -> float"
assert json.loads('"hello"') == "hello", json.loads('"hello"')

print("loads_scalar_type_mapping OK")
"###);
    assert_output(&out, r###"loads_scalar_type_mapping OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/json/mixed_array_roundtrip.py`.
#[test]
fn test_gen_behavior_std_libs_json_mixed_array_roundtrip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "json"
# dimension = "behavior"
# case = "mixed_array_roundtrip"
# subject = "json.dumps"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/json/test_pass1.py"
# status = "filled"
# ///
"""json.dumps: a heterogeneous array of int/str/float/bool/None/dict round-trips through dumps->loads unchanged"""
import json

mixed = [1, "two", 3.0, True, None, {"key": "val"}]
rt = json.loads(json.dumps(mixed))
assert rt == mixed, f"mixed array = {rt!r}"
assert rt[4] is None, rt

print("mixed_array_roundtrip OK")
"###);
    assert_output(&out, r###"mixed_array_roundtrip OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/json/nested_structure_roundtrip.py`.
#[test]
fn test_gen_behavior_std_libs_json_nested_structure_roundtrip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "json"
# dimension = "behavior"
# case = "nested_structure_roundtrip"
# subject = "json.loads"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/json/test_pass1.py"
# status = "filled"
# ///
"""json.loads: a deeply nested dict/list/scalar tree survives dumps->loads byte-for-value round-trip unchanged"""
import json

nested = {"a": {"b": {"c": [1, 2, {"d": 3}]}}}
rt = json.loads(json.dumps(nested))
assert rt == nested, f"nested round-trip = {rt!r}"

deeper = {"outer": {"inner": 42}, "list": [1, 2, {"y": "nested"}]}
back = json.loads(json.dumps(deeper, sort_keys=True))
assert back["outer"]["inner"] == 42, back
assert back["list"][2]["y"] == "nested", back

print("nested_structure_roundtrip OK")
"###);
    assert_output(&out, r###"nested_structure_roundtrip OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/json/separators_compact_output.py`.
#[test]
fn test_gen_behavior_std_libs_json_separators_compact_output() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "json"
# dimension = "behavior"
# case = "separators_compact_output"
# subject = "json.dumps"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/json/test_separators.py"
# status = "filled"
# ///
"""json.dumps: separators=(',',':') drops all inter-token whitespace producing the most compact serialization"""
import json

assert json.dumps([1, 2], separators=(",", ":")) == "[1,2]", json.dumps([1, 2], separators=(",", ":"))
compact = json.dumps({"a": 1, "b": 2}, separators=(",", ":"), sort_keys=True)
assert compact == '{"a":1,"b":2}', f"compact = {compact!r}"

print("separators_compact_output OK")
"###);
    assert_output(&out, r###"separators_compact_output OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/json/sort_keys_orders_object.py`.
#[test]
fn test_gen_behavior_std_libs_json_sort_keys_orders_object() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "json"
# dimension = "behavior"
# case = "sort_keys_orders_object"
# subject = "json.dumps"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/json/test_separators.py"
# status = "filled"
# ///
"""json.dumps: sort_keys=True emits object keys in deterministic ascending order regardless of insertion order"""
import json

assert json.dumps({"b": 2, "a": 1}, sort_keys=True) == '{"a": 1, "b": 2}'
assert json.dumps({"b": 2, "a": 1, "c": 3}, sort_keys=True) == '{"a": 1, "b": 2, "c": 3}'
assert json.dumps({"z": 1, "a": 2, "m": 3}, sort_keys=True) == '{"a": 2, "m": 3, "z": 1}'

print("sort_keys_orders_object OK")
"###);
    assert_output(&out, r###"sort_keys_orders_object OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/json/string_escape_roundtrip.py`.
#[test]
fn test_gen_behavior_std_libs_json_string_escape_roundtrip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "json"
# dimension = "behavior"
# case = "string_escape_roundtrip"
# subject = "json.dumps"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/json/test_scanstring.py"
# status = "filled"
# ///
"""json.dumps: embedded quote, backslash, and newline characters are escaped on dump and restored verbatim on load"""
import json

assert json.dumps('quote"here') == '"quote\\"here"', json.dumps('quote"here')
assert json.dumps("back\\slash") == '"back\\\\slash"', json.dumps("back\\slash")
assert json.loads('"a\\nb"') == "a\nb", json.loads('"a\\nb"')

original = 'tab\there "quote" and \\back\\'
assert json.loads(json.dumps(original)) == original, "escape round-trip"

print("string_escape_roundtrip OK")
"###);
    assert_output(&out, r###"string_escape_roundtrip OK
"###);
}
