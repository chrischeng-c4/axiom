# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_tomllib_loads_ops"
# subject = "cpython321.test_tomllib_loads_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_tomllib_loads_ops.py"
# status = "filled"
# ///
"""cpython321.test_tomllib_loads_ops: execute CPython 3.12 seed test_tomllib_loads_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for the `tomllib` module — the
# Python 3.11+ built-in TOML parser used by `pyproject.toml` /
# `pip` / `uv` / build-backend tools. Surface focuses on
# `tomllib.loads(str)` since `tomllib.load(fp)` requires binary-mode
# open which mamba doesn't yet support. Every probed form returns a
# byte-identical `dict` across mamba and CPython 3.12.
#
# Surface:
#   • tomllib.loads(s: str) → dict
#       — top-level key/value pairs become dict entries;
#       — `[section]` headers become nested dict keys;
#       — `[a.b.c]` deep nesting becomes nested dicts;
#       — `[[item]]` array-of-tables becomes a list of dicts;
#       — primitive types: int (decimal, hex, octal, binary,
#         underscored), float (incl. underscores and scientific
#         notation), bool, str (basic + literal + multi-line), arrays;
#       — inline tables `{x = 1, y = 2}` become dicts;
#       — empty input → empty dict.
import tomllib
_ledger: list[int] = []

# Top-level key/value — string, int, bool, array
r = tomllib.loads('name = "hello"\nversion = "1.0"\n')
assert r["name"] == "hello"; _ledger.append(1)
assert r["version"] == "1.0"; _ledger.append(1)

r = tomllib.loads("port = 8080\n")
assert r["port"] == 8080; _ledger.append(1)
assert isinstance(r["port"], int); _ledger.append(1)

r = tomllib.loads("enabled = true\ndisabled = false\n")
assert r["enabled"] == True; _ledger.append(1)
assert r["disabled"] == False; _ledger.append(1)
assert isinstance(r["enabled"], bool); _ledger.append(1)

# Sections — `[section]` headers
r = tomllib.loads('[section]\nfoo = 42\nbar = true\n')
assert r["section"]["foo"] == 42; _ledger.append(1)
assert r["section"]["bar"] == True; _ledger.append(1)
assert isinstance(r["section"], dict); _ledger.append(1)

# Deep nesting — `[a.b.c]` headers
r = tomllib.loads('[a.b.c]\nkey = "value"\n')
assert r["a"]["b"]["c"]["key"] == "value"; _ledger.append(1)

# Arrays — `[1, 2, 3]` becomes list
r = tomllib.loads("arr = [1, 2, 3]\n")
assert r["arr"] == [1, 2, 3]; _ledger.append(1)
assert isinstance(r["arr"], list); _ledger.append(1)

# String arrays
r = tomllib.loads('deps = ["a", "b", "c"]\n')
assert r["deps"] == ["a", "b", "c"]; _ledger.append(1)

# Nested arrays — `[[1, 2], [3, 4]]`
r = tomllib.loads("nested = [[1, 2], [3, 4]]\n")
assert r["nested"] == [[1, 2], [3, 4]]; _ledger.append(1)

# Array of tables — `[[item]]` becomes list of dicts
r = tomllib.loads("""
[[item]]
name = "first"
value = 1

[[item]]
name = "second"
value = 2
""")
assert isinstance(r["item"], list); _ledger.append(1)
assert len(r["item"]) == 2; _ledger.append(1)
assert r["item"][0]["name"] == "first"; _ledger.append(1)
assert r["item"][1]["name"] == "second"; _ledger.append(1)
assert r["item"][0]["value"] + r["item"][1]["value"] == 3; _ledger.append(1)

# Inline table — `{x = 1, y = 2}` becomes dict
r = tomllib.loads('point = {x = 1, y = 2}\n')
assert r["point"]["x"] == 1; _ledger.append(1)
assert r["point"]["y"] == 2; _ledger.append(1)
assert isinstance(r["point"], dict); _ledger.append(1)

# Empty input — empty dict
assert tomllib.loads("") == {}; _ledger.append(1)
assert isinstance(tomllib.loads(""), dict); _ledger.append(1)

# Multi-line string
r = tomllib.loads('s = """multi\nline"""\n')
assert r["s"] == "multi\nline"; _ledger.append(1)

# Literal string `'literal'` — no escape processing
r = tomllib.loads("s = 'literal'\n")
assert r["s"] == "literal"; _ledger.append(1)

# String escape — `\n` in basic strings becomes newline
r = tomllib.loads('s = "hello\\nworld"\n')
assert r["s"] == "hello\nworld"; _ledger.append(1)

# Negative numbers
r = tomllib.loads("a = -5\nb = -100\n")
assert r["a"] == -5; _ledger.append(1)
assert r["b"] == -100; _ledger.append(1)

# Float — basic and scientific
r = tomllib.loads("a = 3.14\nb = -2.5\nc = 1e10\nd = 1.5e-3\n")
assert r["a"] == 3.14; _ledger.append(1)
assert r["b"] == -2.5; _ledger.append(1)
assert r["c"] == 1e10; _ledger.append(1)
assert r["d"] == 1.5e-3; _ledger.append(1)
assert isinstance(r["a"], float); _ledger.append(1)
assert isinstance(r["c"], float); _ledger.append(1)

# Hex / Octal / Binary integers
r = tomllib.loads("h = 0xff\no = 0o77\nb = 0b101\n")
assert r["h"] == 255; _ledger.append(1)
assert r["o"] == 63; _ledger.append(1)
assert r["b"] == 5; _ledger.append(1)

# Underscored integers — `1_000_000` is 1_000_000
r = tomllib.loads("a = 1_000_000\n")
assert r["a"] == 1_000_000; _ledger.append(1)

# Underscored floats
r = tomllib.loads("a = 1_000.0\n")
assert r["a"] == 1000.0; _ledger.append(1)

# Realistic pyproject-style document
src = """
[package]
name = "test"
version = "1.0.0"
dependencies = ["requests", "click"]

[server]
host = "localhost"
port = 8080
debug = false
"""
r = tomllib.loads(src)
assert r["package"]["name"] == "test"; _ledger.append(1)
assert r["package"]["version"] == "1.0.0"; _ledger.append(1)
assert r["package"]["dependencies"] == ["requests", "click"]; _ledger.append(1)
assert r["server"]["host"] == "localhost"; _ledger.append(1)
assert r["server"]["port"] == 8080; _ledger.append(1)
assert r["server"]["debug"] == False; _ledger.append(1)

# Return-type discipline
assert isinstance(tomllib.loads('a = 1\n'), dict); _ledger.append(1)
assert isinstance(tomllib.loads('a = "x"\n'), dict); _ledger.append(1)
assert isinstance(tomllib.loads('a = 1.5\n'), dict); _ledger.append(1)
assert isinstance(tomllib.loads('a = true\n'), dict); _ledger.append(1)
assert isinstance(tomllib.loads('a = [1,2]\n'), dict); _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_tomllib_loads_ops {sum(_ledger)} asserts")
