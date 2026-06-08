# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_re_module_full_value_ops"
# subject = "cpython321.test_re_module_full_value_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_re_module_full_value_ops.py"
# status = "filled"
# ///
"""cpython321.test_re_module_full_value_ops: execute CPython 3.12 seed test_re_module_full_value_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for the value contract of the
# `re` module-level pattern-matching surface used by every
# regex-driven path: the documented `match` / `search` / `sub`
# / `subn` / `findall` / `split` / `escape` module-level helper
# value contract, the documented `IGNORECASE` / `MULTILINE` /
# `DOTALL` integer-flag-value contract, the documented Match
# object `.group(n)` / `.groups()` / `.start()` / `.end()`
# instance method surface (when constructed by the module-level
# `re.match` / `re.search` helpers), and the documented `match`
# / `search` / `sub` / `subn` / `findall` / `split` / `compile`
# / `escape` / `IGNORECASE` / `MULTILINE` / `DOTALL` / `Pattern`
# / `Match` module hasattr surface.
#
# The matching subset between mamba and CPython is the entire
# module-level helper layer (match / search / sub / subn /
# findall / split / escape with full keyword-arg surface), the
# Match object instance method layer when produced by `re.match`
# / `re.search` (group / groups / start / end), the integer
# flag-value layer (IGNORECASE / MULTILINE / DOTALL coerce to
# the documented int constants), and the full module hasattr
# surface.
#
# Surface in this fixture:
#   • re.split — whitespace-class split + delimiter-class split;
#   • re.sub — count-all replacement + ID-replacement;
#   • re.subn — replacement-count tuple-return;
#   • re.findall — single-group + multi-group capture;
#   • re.match — group(0) / group(n) / groups / start / end on
#     the module-level Match object;
#   • re.search — group(0) / start / end on the module-level
#     Match object;
#   • re.escape — backslash-escape contract;
#   • re flag integer-value contract (IGNORECASE = 2,
#     MULTILINE = 8, DOTALL = 16);
#   • re module hasattr surface (match / search / sub / subn /
#     findall / split / compile / escape / IGNORECASE /
#     MULTILINE / DOTALL / Pattern / Match).
#
# Behavioral edges that DIVERGE on mamba (io.StringIO() returns
# a `dict` — entire instance surface broken — write / getvalue
# cycle returns the empty string and seek AttributeError 'dict'
# object, io.BytesIO() returns a `dict` — write / getvalue
# cycle returns the empty bytes and read AttributeError 'dict'
# object, re.compile(pattern).search(text) returns None on a
# pattern that should match — the compiled-pattern instance
# method surface is broken) are covered in the matching spec
# fixture `lang_io_stringio_recompile_silent`.
import re as _re_mod
from typing import Any

# Module binding retyped as `Any` to bypass Pyright Optional-
# member-access narrowing on the documented `re.match` /
# `re.search` return values — every spec contract below asserts
# a successful match.
re: Any = _re_mod


_ledger: list[int] = []

# 1) re.split — whitespace-class + delimiter-class
assert re.split(r"\s+", "hello   world  foo") == ["hello", "world", "foo"]; _ledger.append(1)
assert re.split(r",", "a,b,c,d") == ["a", "b", "c", "d"]; _ledger.append(1)

# 2) re.sub — replacement contracts
assert re.sub(r"\d+", "X", "abc 123 def 456") == "abc X def X"; _ledger.append(1)
assert re.sub(r"\s+", "_", "a b  c") == "a_b_c"; _ledger.append(1)

# 3) re.subn — replacement-count tuple
assert re.subn(r"\d+", "X", "abc 123 def 456") == ("abc X def X", 2); _ledger.append(1)

# 4) re.findall — single-group + multi-group
assert re.findall(r"\d+", "abc 123 def 456") == ["123", "456"]; _ledger.append(1)
assert re.findall(r"(\w+)=(\d+)", "a=1 b=2 c=3") == [("a", "1"), ("b", "2"), ("c", "3")]; _ledger.append(1)

# 5) re.match — group(n) / groups / start / end
_m = re.match(r"(\w+) (\w+)", "hello world")
assert _m.group(0) == "hello world"; _ledger.append(1)
assert _m.group(1) == "hello"; _ledger.append(1)
assert _m.group(2) == "world"; _ledger.append(1)
assert _m.groups() == ("hello", "world"); _ledger.append(1)
assert _m.start() == 0; _ledger.append(1)
assert _m.end() == 11; _ledger.append(1)

# 6) re.search — group / start / end
_s = re.search(r"\d+", "abc 123 def")
assert _s.group(0) == "123"; _ledger.append(1)
assert _s.start() == 4; _ledger.append(1)
assert _s.end() == 7; _ledger.append(1)

# 7) re.escape — backslash-escape
assert re.escape("a.b*c") == r"a\.b\*c"; _ledger.append(1)
assert re.escape("hello") == "hello"; _ledger.append(1)

# 8) re — integer flag-value contract
assert int(re.IGNORECASE) == 2; _ledger.append(1)
assert int(re.MULTILINE) == 8; _ledger.append(1)
assert int(re.DOTALL) == 16; _ledger.append(1)

# 9) re — module attribute hasattr surface
assert hasattr(re, "match") == True; _ledger.append(1)
assert hasattr(re, "search") == True; _ledger.append(1)
assert hasattr(re, "sub") == True; _ledger.append(1)
assert hasattr(re, "subn") == True; _ledger.append(1)
assert hasattr(re, "findall") == True; _ledger.append(1)
assert hasattr(re, "split") == True; _ledger.append(1)
assert hasattr(re, "compile") == True; _ledger.append(1)
assert hasattr(re, "escape") == True; _ledger.append(1)
assert hasattr(re, "IGNORECASE") == True; _ledger.append(1)
assert hasattr(re, "MULTILINE") == True; _ledger.append(1)
assert hasattr(re, "DOTALL") == True; _ledger.append(1)
assert hasattr(re, "Pattern") == True; _ledger.append(1)
assert hasattr(re, "Match") == True; _ledger.append(1)

# NB: io.StringIO() / io.BytesIO() return `dict` not the
# instance — write / getvalue cycle returns empty and
# seek / read AttributeError on `dict`, re.compile(pattern).
# search(text) returns None for a pattern that should match —
# all DIVERGE on mamba — moved to the divergence-spec fixture.

print(f"MAMBA_ASSERTION_PASS: test_re_module_full_value_ops {sum(_ledger)} asserts")
