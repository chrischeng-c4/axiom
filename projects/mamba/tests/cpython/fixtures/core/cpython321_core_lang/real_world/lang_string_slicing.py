# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_string_slicing"
# subject = "cpython321.lang_string_slicing"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/lang_string_slicing.py"
# status = "filled"
# ///
"""cpython321.lang_string_slicing: execute CPython 3.12 seed lang_string_slicing"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for string indexing and slicing.
# Surface: s[i] yields a one-character string at index i; negative
# indices count from the end; s[a:b] returns the half-open slice
# from a (inclusive) to b (exclusive); omitted bounds default to
# 0 / len; s[::step] strides through the string; s[::-1] reverses
# the string; concatenation with +; repeat with *; `in` / `not in`
# substring tests; len; lexicographic comparison; .join / .split
# round-trip; .split with maxsplit caps the number of cuts.
_ledger: list[int] = []

s = "hello world"

# Positive index: first and last
assert s[0] == "h"; _ledger.append(1)
assert s[4] == "o"; _ledger.append(1)

# Negative index counts from the end (-1 is last char)
assert s[-1] == "d"; _ledger.append(1)
assert s[-5] == "w"; _ledger.append(1)

# Half-open slice [a:b) — start inclusive, stop exclusive
assert s[0:5] == "hello"; _ledger.append(1)
assert s[6:11] == "world"; _ledger.append(1)

# Omitted start defaults to 0
assert s[:5] == "hello"; _ledger.append(1)
# Omitted stop defaults to len(s)
assert s[6:] == "world"; _ledger.append(1)
# Both omitted is a copy
assert s[:] == "hello world"; _ledger.append(1)

# Negative-only slice from the end
assert s[-5:] == "world"; _ledger.append(1)
# Negative stop excludes the last n chars
assert s[1:-1] == "ello worl"; _ledger.append(1)

# Step argument strides through the string
assert s[::2] == "hlowrd"; _ledger.append(1)
# Negative step reverses the string
assert s[::-1] == "dlrow olleh"; _ledger.append(1)

# Concatenation with +
assert "ab" + "cd" == "abcd"; _ledger.append(1)
# Repeat with *
assert "ab" * 3 == "ababab"; _ledger.append(1)
# Repeat by zero yields empty
assert "ab" * 0 == ""; _ledger.append(1)

# `in` substring containment
assert ("ell" in "hello") == True; _ledger.append(1)
assert ("xyz" in "hello") == False; _ledger.append(1)
# `not in` is the inverse
assert ("xyz" not in "hello") == True; _ledger.append(1)
assert ("ell" not in "hello") == False; _ledger.append(1)

# len returns the number of characters
assert len("hello") == 5; _ledger.append(1)
assert len("") == 0; _ledger.append(1)

# Lexicographic comparison: shorter prefix is less than longer
assert ("abc" == "abc") == True; _ledger.append(1)
assert ("abc" < "abd") == True; _ledger.append(1)
assert ("abc" > "abb") == True; _ledger.append(1)
assert ("ab" < "abc") == True; _ledger.append(1)

# .join with a separator and .split round-trip
assert "-".join(["a", "b", "c"]) == "a-b-c"; _ledger.append(1)
assert "a,b,c".split(",") == ["a", "b", "c"]; _ledger.append(1)
# Splitting on a separator that isn't present yields the original as one item
assert "abc".split("x") == ["abc"]; _ledger.append(1)

# .split with maxsplit caps the number of cuts; the tail joins the rest
assert "a,b,c,d".split(",", 2) == ["a", "b", "c,d"]; _ledger.append(1)
# maxsplit=0 returns the original as one item
assert "a,b,c".split(",", 0) == ["a,b,c"]; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: lang_string_slicing {sum(_ledger)} asserts")
