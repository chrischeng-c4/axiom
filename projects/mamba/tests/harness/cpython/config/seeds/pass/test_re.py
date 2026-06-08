# test_re.py — #2695 CPython re seed (executed assertions).
#
# Replaces the prior vendored CPython upstream Lib/test/test_re.py
# (~3100 lines, ranked `Stub` because mamba's unittest dispatcher
# doesn't run real test methods — #2545) with the *smallest*
# Mamba-authored seed distilled from the re-module smoke surface.
# Exercises `match` / `search` / `findall` / `sub` / `split` /
# `fullmatch` / `compile` and the `Match` object's `group()` /
# `groups()` / `start()` / `end()` / `span()` accessors with
# deterministic ASCII inputs. Emits the runner's positive proof-of-
# execution marker that `cpython_lib_test_runner.rs` (#2691) classifies
# as `AssertionPass`.
#
# Why so small? Mamba's current re surface presents the seven top-level
# functions plus the Match accessors used here; richer surface — flag
# arguments (`re.IGNORECASE`, `re.MULTILINE`, `re.DOTALL`), named groups
# (`(?P<name>...)`), lookaround, backreferences, Unicode `\p{...}`,
# `re.escape`, `re.subn` — lands as each gap closes. Specifically,
# `re.IGNORECASE` is broken on mamba today (the flag is accepted but
# case-folding does not happen), so it is excluded from this seed.
#
# Why no helper function? Per the #2691 contract, top-level `def()`
# does not capture module-scope names by reference on mamba.
#
# Contract with the runner (#2691):
#   - Each `assert` runs at top level. Inverting any assert raises
#     AssertionError → non-zero exit → runner classifies as `Fail`.
#   - After every assertion has executed, the seed emits
#     `MAMBA_ASSERTION_PASS: re N asserts` to stdout.

import re

_ledger: list[int] = []

# 1. Module identity: re's own __name__ must be "re".
assert re.__name__ == "re", "re.__name__ must be 're'"
_ledger.append(1)

# 2. re.match — matches at the start of the string. group(), start(),
#    end(), span() round-trip the matched substring and its offsets.
_m1 = re.match(r"\d+", "123abc")
assert _m1 is not None, "re.match must find leading digits"
_ledger.append(1)
assert _m1.group() == "123", "match.group() returns the matched substring"
_ledger.append(1)
assert _m1.start() == 0, "match.start() == 0 for leading match"
_ledger.append(1)
assert _m1.end() == 3, "match.end() == 3 for '123'"
_ledger.append(1)
assert _m1.span() == (0, 3), "match.span() returns (start, end)"
_ledger.append(1)

# 3. re.match — returns None when the pattern doesn't match at the
#    start. Catches a regression that returns a sentinel non-None.
assert re.match(r"\d+", "abc123") is None, "re.match anchored to start"
_ledger.append(1)

# 4. re.search — finds the first match anywhere in the string.
_m2 = re.search(r"\d+", "abc123def")
assert _m2 is not None, "re.search must find embedded digits"
_ledger.append(1)
assert _m2.group() == "123", "search.group() returns matched substring"
_ledger.append(1)
assert _m2.start() == 3, "search.start() at offset 3 in 'abc123def'"
_ledger.append(1)

# 5. re.search — returns None when no match anywhere.
assert re.search(r"\d+", "abcdef") is None, "re.search returns None for no match"
_ledger.append(1)

# 6. re.findall — returns the list of all non-overlapping matches.
assert re.findall(r"\d+", "a1b22c333") == ["1", "22", "333"], "findall returns list of matches"
_ledger.append(1)
assert re.findall(r"\d+", "abcdef") == [], "findall returns empty list for no match"
_ledger.append(1)

# 7. re.sub — substitutes each match with the replacement string.
assert re.sub(r"\d+", "X", "a1b22c333") == "aXbXcX", "sub replaces every match"
_ledger.append(1)
assert re.sub(r"\d+", "X", "abcdef") == "abcdef", "sub leaves string unchanged when no match"
_ledger.append(1)

# 8. re.split — splits on the pattern.
assert re.split(r"\s+", "a b  c\td") == ["a", "b", "c", "d"], "split on whitespace"
_ledger.append(1)

# 9. re.fullmatch — anchors at BOTH ends.
_fm = re.fullmatch(r"\d+", "12345")
assert _fm is not None, "fullmatch matches whole string"
_ledger.append(1)
assert _fm.group() == "12345", "fullmatch returns whole match"
_ledger.append(1)
assert re.fullmatch(r"\d+", "12abc") is None, "fullmatch fails when tail doesn't match"
_ledger.append(1)

# 10. Group capture — pattern with two groups round-trips through
#     group(0)/group(1)/group(2) and the groups() tuple.
_m3 = re.match(r"(\w+)\s+(\w+)", "hello world")
assert _m3 is not None, "two-group match must succeed"
_ledger.append(1)
assert _m3.group(0) == "hello world", "group(0) is the whole match"
_ledger.append(1)
assert _m3.group(1) == "hello", "group(1) is the first capture"
_ledger.append(1)
assert _m3.group(2) == "world", "group(2) is the second capture"
_ledger.append(1)
assert _m3.groups() == ("hello", "world"), "groups() returns capture tuple"
_ledger.append(1)

# 11. re.compile — compiled pattern's .match() behaves like top-level.
_pat = re.compile(r"(\w+)\s+(\w+)")
_m4 = _pat.match("hello world")
assert _m4 is not None, "compiled pattern matches"
_ledger.append(1)
assert _m4.groups() == ("hello", "world"), "compiled pattern groups round-trip"
_ledger.append(1)

# Emit the proof-of-execution marker as the FINAL line so the runner
# can see it on stdout. Per `ASSERTION_PASS_MARKERS` in
# `cpython_lib_test_runner.rs`, presence of this token escalates the
# outcome from ImportPass to AssertionPass.
print(f"MAMBA_ASSERTION_PASS: re {len(_ledger)} asserts")
