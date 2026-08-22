use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/tomllib/array_of_tables_to_list.py`.
#[test]
fn test_gen_behavior_std_libs_tomllib_array_of_tables_to_list() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tomllib"
# dimension = "behavior"
# case = "array_of_tables_to_list"
# subject = "tomllib.loads"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tomllib/test_data.py"
# status = "filled"
# ///
"""tomllib.loads: repeated [[products]] headers build a Python list of dicts in document order, preserving each table's keys"""
import tomllib

_d = tomllib.loads("""
[[products]]
name = "Hammer"
price = 9.99

[[products]]
name = "Wrench"
price = 14.99
""")
assert isinstance(_d["products"], list), f"array of tables type = {type(_d['products'])!r}"
assert len(_d["products"]) == 2, f"two products = {len(_d['products'])!r}"
assert _d["products"][0]["name"] == "Hammer", f"first = {_d['products'][0]['name']!r}"
assert _d["products"][1]["name"] == "Wrench", f"second = {_d['products'][1]['name']!r}"

print("array_of_tables_to_list OK")
"###);
    assert_output(&out, r###"array_of_tables_to_list OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/tomllib/basic_vs_literal_string_escapes.py`.
#[test]
fn test_gen_behavior_std_libs_tomllib_basic_vs_literal_string_escapes() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tomllib"
# dimension = "behavior"
# case = "basic_vs_literal_string_escapes"
# subject = "tomllib.loads"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tomllib/test_data.py"
# status = "filled"
# ///
"""tomllib.loads: basic strings process backslash escapes (\\n -> newline); literal single-quoted strings keep the backslash literal"""
import tomllib

_toml = (
    'basic = "hello\\nworld"\n'
    "literal = 'no\\escape'\n"  # literal: backslash stays literal
)
_d = tomllib.loads(_toml)
assert _d["basic"] == "hello\nworld", f"basic string escape = {_d['basic']!r}"
assert _d["literal"] == "no\\escape", f"literal no escape = {_d['literal']!r}"

print("basic_vs_literal_string_escapes OK")
"###);
    assert_output(&out, r###"basic_vs_literal_string_escapes OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/tomllib/boolean_literals.py`.
#[test]
fn test_gen_behavior_std_libs_tomllib_boolean_literals() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tomllib"
# dimension = "behavior"
# case = "boolean_literals"
# subject = "tomllib.loads"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tomllib/test_data.py"
# status = "filled"
# ///
"""tomllib.loads: the literals true and false parse to the Python singletons True and False"""
import tomllib

_d = tomllib.loads("a = true\nb = false")
assert _d["a"] is True, f"true = {_d['a']!r}"
assert _d["b"] is False, f"false = {_d['b']!r}"

print("boolean_literals OK")
"###);
    assert_output(&out, r###"boolean_literals OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/tomllib/datetime_date_time_types.py`.
#[test]
fn test_gen_behavior_std_libs_tomllib_datetime_date_time_types() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tomllib"
# dimension = "behavior"
# case = "datetime_date_time_types"
# subject = "tomllib.loads"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tomllib/test_misc.py"
# status = "filled"
# ///
"""tomllib.loads: offset datetime parses to datetime.datetime, a bare date to datetime.date, a bare time to datetime.time, with correct year/month fields"""
import tomllib
import datetime

_d = tomllib.loads("""
dt = 2023-01-15T10:30:00Z
date_only = 2023-01-15
time_only = 10:30:00
""")
assert isinstance(_d["dt"], datetime.datetime), f"datetime type = {type(_d['dt'])!r}"
assert isinstance(_d["date_only"], datetime.date), f"date type = {type(_d['date_only'])!r}"
assert isinstance(_d["time_only"], datetime.time), f"time type = {type(_d['time_only'])!r}"
assert _d["dt"].year == 2023, f"year = {_d['dt'].year!r}"
assert _d["dt"].month == 1, f"month = {_d['dt'].month!r}"
assert _d["dt"].day == 15, f"day = {_d['dt'].day!r}"

print("datetime_date_time_types OK")
"###);
    assert_output(&out, r###"datetime_date_time_types OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/tomllib/deep_nested_tables.py`.
#[test]
fn test_gen_behavior_std_libs_tomllib_deep_nested_tables() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tomllib"
# dimension = "behavior"
# case = "deep_nested_tables"
# subject = "tomllib.loads"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tomllib/test_data.py"
# status = "filled"
# ///
"""tomllib.loads: a dotted [a.b.c] header builds nested dicts so data['a']['b']['c']['value'] resolves"""
import tomllib

_d = tomllib.loads("""
[a.b.c]
value = "deep"
""")
assert _d["a"]["b"]["c"]["value"] == "deep", f"deep nesting = {_d!r}"

print("deep_nested_tables OK")
"###);
    assert_output(&out, r###"deep_nested_tables OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/tomllib/float_forms_and_special.py`.
#[test]
fn test_gen_behavior_std_libs_tomllib_float_forms_and_special() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tomllib"
# dimension = "behavior"
# case = "float_forms_and_special"
# subject = "tomllib.loads"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tomllib/test_data.py"
# status = "filled"
# ///
"""tomllib.loads: floats parse plain (3.14159), negative, exponent (6.022e23, 1e-10), and the special values inf and nan"""
import tomllib
import math

_d = tomllib.loads("""
pi = 3.14159
neg = -2.5
exp = 6.022e23
small = 1e-10
inf = inf
nan = nan
""")
assert abs(_d["pi"] - 3.14159) < 1e-10, f"pi = {_d['pi']!r}"
assert _d["neg"] == -2.5, f"neg = {_d['neg']!r}"
assert abs(_d["exp"] - 6.022e23) / 6.022e23 < 1e-10, f"exp = {_d['exp']!r}"
assert abs(_d["small"] - 1e-10) < 1e-20, f"small = {_d['small']!r}"
assert math.isinf(_d["inf"]) and _d["inf"] > 0, f"inf = {_d['inf']!r}"
assert math.isnan(_d["nan"]), f"nan = {_d['nan']!r}"

print("float_forms_and_special OK")
"###);
    assert_output(&out, r###"float_forms_and_special OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/tomllib/inline_table_to_dict.py`.
#[test]
fn test_gen_behavior_std_libs_tomllib_inline_table_to_dict() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tomllib"
# dimension = "behavior"
# case = "inline_table_to_dict"
# subject = "tomllib.loads"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tomllib/test_data.py"
# status = "filled"
# ///
"""tomllib.loads: an inline table point = {x = 1, y = 2} parses to the Python dict {'x': 1, 'y': 2}"""
import tomllib

_d = tomllib.loads('point = {x = 1, y = 2}')
assert _d["point"] == {"x": 1, "y": 2}, f"inline table = {_d['point']!r}"

print("inline_table_to_dict OK")
"###);
    assert_output(&out, r###"inline_table_to_dict OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/tomllib/integer_radix_and_underscores.py`.
#[test]
fn test_gen_behavior_std_libs_tomllib_integer_radix_and_underscores() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tomllib"
# dimension = "behavior"
# case = "integer_radix_and_underscores"
# subject = "tomllib.loads"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tomllib/test_data.py"
# status = "filled"
# ///
"""tomllib.loads: integers parse in decimal, hex (0xFF=255), octal (0o77=63), binary (0b1010=10), signed, and with underscore separators (1_000_000)"""
import tomllib

_d = tomllib.loads("""
decimal = 42
negative = -17
hex_val = 0xFF
octal_val = 0o77
binary_val = 0b1010
underscore = 1_000_000
""")
assert _d["decimal"] == 42, f"decimal = {_d['decimal']!r}"
assert _d["negative"] == -17, f"negative = {_d['negative']!r}"
assert _d["hex_val"] == 255, f"hex = {_d['hex_val']!r}"
assert _d["octal_val"] == 63, f"octal = {_d['octal_val']!r}"
assert _d["binary_val"] == 10, f"binary = {_d['binary_val']!r}"
assert _d["underscore"] == 1000000, f"underscore = {_d['underscore']!r}"

print("integer_radix_and_underscores OK")
"###);
    assert_output(&out, r###"integer_radix_and_underscores OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/tomllib/load_from_binary_file.py`.
#[test]
fn test_gen_behavior_std_libs_tomllib_load_from_binary_file() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tomllib"
# dimension = "behavior"
# case = "load_from_binary_file"
# subject = "tomllib.load"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tomllib/test_misc.py"
# status = "filled"
# ///
"""tomllib.load: tomllib.load reads from a binary file object (BytesIO and an on-disk tempfile opened 'rb') and yields the same dict tomllib.loads would"""
import tomllib
import io
import os
import tempfile

_content = b'key = "value"\nnum = 42\n'
_expected = {"key": "value", "num": 42}

# In-memory binary file object.
_d_mem = tomllib.load(io.BytesIO(_content))
assert _d_mem == _expected, f"BytesIO load = {_d_mem!r}"

# On-disk file opened in binary mode, inside a TemporaryDirectory.
with tempfile.TemporaryDirectory() as _tmp:
    _path = os.path.join(_tmp, "config.toml")
    with open(_path, "wb") as _wf:
        _wf.write(_content)
    with open(_path, "rb") as _rf:
        _d_disk = tomllib.load(_rf)
assert _d_disk == _expected, f"file load = {_d_disk!r}"

# Both paths agree with the string parser.
assert _d_mem == tomllib.loads(_content.decode()), "load vs loads divergence"

print("load_from_binary_file OK")
"###);
    assert_output(&out, r###"load_from_binary_file OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/tomllib/mixed_and_nested_arrays.py`.
#[test]
fn test_gen_behavior_std_libs_tomllib_mixed_and_nested_arrays() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tomllib"
# dimension = "behavior"
# case = "mixed_and_nested_arrays"
# subject = "tomllib.loads"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tomllib/test_data.py"
# status = "filled"
# ///
"""tomllib.loads: TOML 1.0 arrays hold mixed types ([1, 2.0, 'three']) and nest ([[1,2],[3,4]]) into the corresponding Python list structure"""
import tomllib

_d = tomllib.loads("""
mixed = [1, 2.0, "three"]
nested = [[1, 2], [3, 4]]
""")
assert _d["mixed"][0] == 1, f"mixed[0] = {_d['mixed'][0]!r}"
assert _d["mixed"][1] == 2.0, f"mixed[1] = {_d['mixed'][1]!r}"
assert _d["mixed"][2] == "three", f"mixed[2] = {_d['mixed'][2]!r}"
assert _d["nested"] == [[1, 2], [3, 4]], f"nested = {_d['nested']!r}"

print("mixed_and_nested_arrays OK")
"###);
    assert_output(&out, r###"mixed_and_nested_arrays OK
"###);
}
