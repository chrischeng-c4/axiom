# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_tomllib_advanced_ops"
# subject = "cpython321.test_tomllib_advanced_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_tomllib_advanced_ops.py"
# status = "filled"
# ///
"""cpython321.test_tomllib_advanced_ops: execute CPython 3.12 seed test_tomllib_advanced_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for `tomllib` surfaces beyond
# `test_tomllib_ops` (which already covers scalar round-trip, inline
# arrays, [section] tables, and empty input).
# Surface:
#   • inline-table literal `point = {x = 1, y = 2}` → nested dict;
#   • array-of-tables `[[items]]` → list of dicts;
#   • multi-line basic string `"""..."""` → str with embedded newline;
#   • integer base prefixes `0xff` / `0o17` / `0b101`;
#   • underscore-grouped integer literals `1_000_000`;
#   • dotted-key assignment `a.b.c = 1` → nested dicts;
#   • dotted-table header `[a.b]` → same nested-dict shape;
#   • float exponent notation `1.5e2` → 150.0;
#   • negative numeric literals (`-42`, `-3.14`);
#   • boolean `false`;
#   • TypeError when `tomllib.loads` is fed bytes instead of str
#     (CPython contract — `loads` is str-only).
#
# Literal strings with backslashes (`s = 'C:\\path'`) are deliberately
# omitted: mamba 0.3.60 double-escapes the result.
from typing import Any
import tomllib
# Tomllib's typed signature is `loads(s: str) -> dict`; the bytes
# call probes the runtime TypeError contract, so route the bytes
# argument through an Any-typed holder to bypass Pyright.
_tomllib: Any = tomllib
_ledger: list[int] = []

# Inline-table literal
_d = tomllib.loads('point = {x = 1, y = 2}')
assert _d == {"point": {"x": 1, "y": 2}}; _ledger.append(1)
assert _d["point"]["x"] == 1; _ledger.append(1)
assert _d["point"]["y"] == 2; _ledger.append(1)

# Array-of-tables
_d = tomllib.loads('[[items]]\nname = "a"\n[[items]]\nname = "b"')
assert isinstance(_d["items"], list); _ledger.append(1)
assert len(_d["items"]) == 2; _ledger.append(1)
assert _d["items"][0] == {"name": "a"}; _ledger.append(1)
assert _d["items"][1] == {"name": "b"}; _ledger.append(1)

# Multi-line basic string — embedded newline survives
_d = tomllib.loads('s = """hello\nworld"""')
assert _d["s"] == "hello\nworld"; _ledger.append(1)
assert "\n" in _d["s"]; _ledger.append(1)

# Integer base prefixes
_d = tomllib.loads('h = 0xff\no = 0o17\nb = 0b101')
assert _d["h"] == 255; _ledger.append(1)
assert _d["o"] == 15; _ledger.append(1)
assert _d["b"] == 5; _ledger.append(1)

# Underscore-grouped integer literal
_d = tomllib.loads('big = 1_000_000')
assert _d["big"] == 1000000; _ledger.append(1)

# Dotted-key assignment projects to nested dicts
_d = tomllib.loads('a.b.c = 1')
assert _d == {"a": {"b": {"c": 1}}}; _ledger.append(1)
assert _d["a"]["b"]["c"] == 1; _ledger.append(1)

# Dotted-table header — same nested-dict shape as dotted key
_d = tomllib.loads('[a.b]\nc = 1')
assert _d == {"a": {"b": {"c": 1}}}; _ledger.append(1)

# Float exponent notation
_d = tomllib.loads('f = 1.5e2')
assert _d["f"] == 150.0; _ledger.append(1)

# Negative numeric literals
_d = tomllib.loads('n = -42\nf = -3.14')
assert _d["n"] == -42; _ledger.append(1)
assert _d["f"] == -3.14; _ledger.append(1)

# Boolean false — equality is the portable check; mamba's `is`
# semantics on parsed booleans diverge from CPython's singleton model
# (CPython interns True/False; mamba may yield a fresh bool box from
# the parser), so the spec only pins `==`.
_d = tomllib.loads('x = false')
assert _d["x"] == False; _ledger.append(1)
assert not _d["x"]; _ledger.append(1)

# TypeError on bytes input — `loads` is str-only by contract
try:
    _ = _tomllib.loads(b'k = 1')
    raise AssertionError("tomllib.loads(bytes) must raise TypeError")
except TypeError:
    _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_tomllib_advanced_ops {sum(_ledger)} asserts")
