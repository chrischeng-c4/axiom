# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "cpython321_builtins"
# dimension = "real_world"
# case = "test_str_methods_extras_ops"
# subject = "cpython321.test_str_methods_extras_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_str_methods_extras_ops.py"
# status = "filled"
# ///
"""cpython321.test_str_methods_extras_ops: execute CPython 3.12 seed test_str_methods_extras_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for str methods that the other
# test_str_*_ops fixtures don't already cover.
# Surface: .casefold (ASCII + German ß folds to ss), .expandtabs with
# explicit tabsize, .splitlines with and without keepends, .zfill on
# unsigned and negative-prefixed numbers, .center with explicit fill,
# .find returning -1 on miss, .rfind picking the last match, .count
# of a substring (non-overlapping semantics), .replace with a count
# limit, printf-style `%` formatting (%d %s, zero-pad width, fixed
# float precision), and .format() with positional and keyword refs.
_ledger: list[int] = []

# casefold lowercases more aggressively than .lower()
assert "ABC".casefold() == "abc"; _ledger.append(1)
# German sharp s folds to "ss" under casefold but not under .lower()
assert "Straße".casefold() == "strasse"; _ledger.append(1)

# expandtabs(n) expands each tab to the next multiple of n
assert "a\tb\tc".expandtabs(4) == "a   b   c"; _ledger.append(1)

# splitlines splits on every recognized line ending and drops them
assert "a\nb\r\nc".splitlines() == ["a", "b", "c"]; _ledger.append(1)
# splitlines(keepends=True) preserves the line ending bytes
assert "a\nb\r\nc".splitlines(keepends=True) == ["a\n", "b\r\n", "c"]; _ledger.append(1)

# zfill pads with leading zeros to the requested width
assert "42".zfill(6) == "000042"; _ledger.append(1)
# zfill is sign-aware: the leading - or + stays in front of the zeros
assert "-42".zfill(6) == "-00042"; _ledger.append(1)

# center(width, fill) centers the value with explicit fill character
assert "x".center(7, "-") == "---x---"; _ledger.append(1)

# find returns -1 when the substring isn't present
assert "abc".find("z") == -1; _ledger.append(1)
# rfind returns the index of the LAST occurrence
assert "abcabc".rfind("b") == 4; _ledger.append(1)

# count finds non-overlapping occurrences of a substring
assert "aaaa".count("aa") == 2; _ledger.append(1)

# replace(old, new, count) stops after count substitutions
assert "aaaa".replace("a", "b", 2) == "bbaa"; _ledger.append(1)

# Printf-style %s and %d substitution
assert "%d-%s" % (5, "x") == "5-x"; _ledger.append(1)
# Printf-style zero-pad width
assert "%05d" % 42 == "00042"; _ledger.append(1)
# Printf-style fixed float precision
assert "%.2f" % 3.14159 == "3.14"; _ledger.append(1)

# .format() positional reference: {0}, {1} can repeat
assert "{0}-{1}-{0}".format("a", "b") == "a-b-a"; _ledger.append(1)
# .format() keyword reference: {name}
assert "{name}".format(name="Alice") == "Alice"; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: test_str_methods_extras_ops {sum(_ledger)} asserts")
