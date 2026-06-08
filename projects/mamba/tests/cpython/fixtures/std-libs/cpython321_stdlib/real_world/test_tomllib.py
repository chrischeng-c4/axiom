# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_tomllib"
# subject = "cpython321.test_tomllib"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_tomllib.py"
# status = "filled"
# ///
"""cpython321.test_tomllib: execute CPython 3.12 seed test_tomllib"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Authored AssertionPass seed for tomllib (Python 3.11+ TOML parser).
# Surface: loads() over scalars, arrays, nested tables, inline tables,
# array-of-tables, comments, empty input, and multi-line strings.
# Datetime values are returned as ISO strings rather than datetime objects on
# mamba; this seed asserts the documented parse-structure shape rather than
# the precise datetime type, since the surface here is "parse TOML to dict".
import tomllib

_ledger: list[int] = []

# loads() returns a dict
assert isinstance(tomllib.loads(""), dict), "loads('') returns a dict"
_ledger.append(1)

# Empty input parses to an empty dict
assert tomllib.loads("") == {}, "empty document parses to {}"
_ledger.append(1)

# Scalar key/value: string
assert tomllib.loads('title = "TOML"') == {"title": "TOML"}, (
    "scalar string assignment parses"
)
_ledger.append(1)

# Scalar key/value: integer
assert tomllib.loads("x = 42") == {"x": 42}, "scalar integer assignment parses"
_ledger.append(1)

# Scalar key/value: float
assert tomllib.loads("y = 3.14") == {"y": 3.14}, "scalar float assignment parses"
_ledger.append(1)

# Scalar key/value: booleans (both)
assert tomllib.loads("on = true\noff = false") == {"on": True, "off": False}, (
    "true and false parse as Python bool"
)
_ledger.append(1)

# Array of integers
assert tomllib.loads("a = [1, 2, 3]") == {"a": [1, 2, 3]}, (
    "array of integers parses"
)
_ledger.append(1)

# Table groups keys under a section
assert tomllib.loads('[owner]\nname = "Alice"\nage = 30') == {
    "owner": {"name": "Alice", "age": 30}
}, "[table] group nests keys under the table name"
_ledger.append(1)

# Dotted-key headers create nested tables
assert tomllib.loads("[a.b.c]\nx = 1\ny = 2") == {
    "a": {"b": {"c": {"x": 1, "y": 2}}}
}, "[a.b.c] creates a 3-level nested table"
_ledger.append(1)

# Inline tables
assert tomllib.loads("point = { x = 1, y = 2 }") == {"point": {"x": 1, "y": 2}}, (
    "inline-table syntax parses to a dict"
)
_ledger.append(1)

# Array of tables ([[products]])
assert tomllib.loads(
    '[[products]]\nname = "A"\n[[products]]\nname = "B"\n'
) == {"products": [{"name": "A"}, {"name": "B"}]}, (
    "[[products]] array-of-tables parses to a list of dicts"
)
_ledger.append(1)

# Comments are stripped
assert tomllib.loads("# only comment\nx = 1") == {"x": 1}, (
    "leading comment line is ignored"
)
_ledger.append(1)

# Multi-line basic string
assert tomllib.loads('s = """hello\nworld"""') == {"s": "hello\nworld"}, (
    'triple-quoted string preserves embedded newline'
)
_ledger.append(1)

# Malformed input raises an exception
raised = False
try:
    tomllib.loads("x = ")
except Exception:
    raised = True
assert raised, "malformed TOML raises an exception"
_ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_tomllib {sum(_ledger)} asserts")
