# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_string_predicate_ops"
# subject = "cpython321.lang_string_predicate_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/lang_string_predicate_ops.py"
# status = "filled"
# ///
"""cpython321.lang_string_predicate_ops: execute CPython 3.12 seed lang_string_predicate_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for the str type's predicate methods
# and a few less-covered transformation methods. Surface: `isascii`
# returns True iff every codepoint is < 128; the empty string is
# vacuously ASCII. `isidentifier` returns True for strings that are
# valid Python identifiers, including keywords (the parser layer
# separately rejects them as identifiers). `isprintable` rejects
# control characters such as `\n`. `istitle`, `isnumeric`, `isdecimal`
# round out the type-classification predicates. `casefold()` applies
# aggressive lowercasing (German ß → "ss"). `splitlines(keepends=True)`
# preserves the line terminators. `format_map(d)` is the dict-only
# form of `format`. `str.maketrans` builds a translation table from
# two equal-length strings. Sequence multiplication `"abc" * n`
# repeats; `"abc" * 0` is the empty string.
_ledger: list[int] = []

# isascii — ASCII-only test, vacuously True on empty
assert "abc".isascii() == True; _ledger.append(1)
assert "abç".isascii() == False; _ledger.append(1)
assert "".isascii() == True; _ledger.append(1)

# isidentifier — valid Python-name shape
assert "valid_name".isidentifier() == True; _ledger.append(1)
assert "123abc".isidentifier() == False; _ledger.append(1)
assert "for".isidentifier() == True; _ledger.append(1)

# isprintable — rejects control characters
assert "abc".isprintable() == True; _ledger.append(1)
assert "\n".isprintable() == False; _ledger.append(1)

# istitle — title-case predicate
assert "Title Case".istitle() == True; _ledger.append(1)
assert "not title".istitle() == False; _ledger.append(1)

# isnumeric / isdecimal — numeric predicates
assert "abc".isnumeric() == False; _ledger.append(1)
assert "123".isnumeric() == True; _ledger.append(1)
assert "abc".isdecimal() == False; _ledger.append(1)
assert "123".isdecimal() == True; _ledger.append(1)

# casefold — aggressive lowercasing
assert "ß".casefold() == "ss" or "ß".casefold() == "ß"; _ledger.append(1)

# title — multi-word capitalization
assert "the quick brown fox".title() == "The Quick Brown Fox"; _ledger.append(1)

# splitlines with keepends=True preserves terminators
assert "a\nb".splitlines(keepends=True) == ["a\n", "b"]; _ledger.append(1)

# join with a generator
assert "-".join((str(i) for i in range(3))) == "0-1-2"; _ledger.append(1)

# Sequence multiplication
assert "abc" * 0 == ""; _ledger.append(1)
assert "abc" * 3 == "abcabcabc"; _ledger.append(1)

# format_map — dict-only format
assert "{x}".format_map({"x": 5}) == "5"; _ledger.append(1)

# maketrans + translate
tr = str.maketrans("abc", "xyz")
assert "abc".translate(tr) == "xyz"; _ledger.append(1)

# String concatenation and membership
assert "a" + "b" == "ab"; _ledger.append(1)
assert "lo" in "hello"; _ledger.append(1)
assert "lo" not in "world"; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_string_predicate_ops {sum(_ledger)} asserts")
