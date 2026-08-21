# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "cpython321_builtins"
# dimension = "real_world"
# case = "test_re_subn_finditer_ops"
# subject = "cpython321.test_re_subn_finditer_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_re_subn_finditer_ops.py"
# status = "filled"
# ///
"""cpython321.test_re_subn_finditer_ops: execute CPython 3.12 seed test_re_subn_finditer_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for `re.subn`, `re.finditer`, and
# `re.escape` corners not already covered by `test_re_ops`,
# `test_re_advanced_ops`, `test_re_flags_ops`, or
# `test_re_named_groups_ops`. Surface: `re.subn(pattern, repl, text)`
# returns the `(new_text, replacement_count)` tuple; the count tracks
# the number of distinct substitutions applied. `re.finditer` yields
# one match object per non-overlapping occurrence; `len(list(...))`
# of the iterator equals the expected hit count. `re.escape` quotes
# regex metacharacters and is a no-op on plain alphanumerics.
# `re.compile(pat).match(text)` mirrors the top-level `re.match`
# surface and exposes `.group(n)` for indexed captures.
import re
_ledger: list[int] = []

# subn returns (new_text, count)
result, count = re.subn(r"\d", "X", "a1b2c3")
assert result == "aXbXcX"; _ledger.append(1)
assert count == 3; _ledger.append(1)

# subn with no matches
result_zero, count_zero = re.subn(r"\d", "X", "abc")
assert result_zero == "abc"; _ledger.append(1)
assert count_zero == 0; _ledger.append(1)

# subn with single hit
result_one, count_one = re.subn(r"\d", "X", "a1bc")
assert result_one == "aXbc"; _ledger.append(1)
assert count_one == 1; _ledger.append(1)

# finditer hit count
matches = list(re.finditer(r"\d", "a1b2c3"))
assert len(matches) == 3; _ledger.append(1)

matches_zero = list(re.finditer(r"\d", "abc"))
assert len(matches_zero) == 0; _ledger.append(1)

# escape: metacharacters quoted
assert re.escape("a.b*c") == r"a\.b\*c"; _ledger.append(1)
# escape: alphanumerics unchanged
assert re.escape("hello") == "hello"; _ledger.append(1)
assert re.escape("abc123") == "abc123"; _ledger.append(1)

# compile + match + group
pat = re.compile(r"(\w+)@(\w+)")
m = pat.match("user@host")
assert m is not None; _ledger.append(1)
assert m.group(1) == "user"; _ledger.append(1)
assert m.group(2) == "host"; _ledger.append(1)

# fullmatch anchors at both ends
assert re.fullmatch(r"\d+", "123") is not None; _ledger.append(1)
assert re.fullmatch(r"\d+", "123abc") is None; _ledger.append(1)
assert re.fullmatch(r"\d+", "abc123") is None; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_re_subn_finditer_ops {sum(_ledger)} asserts")
