# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_fnmatch"
# subject = "cpython321.test_fnmatch"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_fnmatch.py"
# status = "filled"
# ///
"""cpython321.test_fnmatch: execute CPython 3.12 seed test_fnmatch"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
import fnmatch

_ledger: list[int] = []

# Basic positive match
assert fnmatch.fnmatch("test.py", "*.py"), "*.py matches test.py"
_ledger.append(1)

# Basic negative match
assert not fnmatch.fnmatch("test.py", "*.txt"), "*.txt does not match test.py"
_ledger.append(1)

# Single-char wildcard
assert fnmatch.fnmatch("abc", "a?c"), "a?c matches abc"
_ledger.append(1)

# Character class
assert fnmatch.fnmatch("abc", "[abc]bc"), "[abc]bc matches abc"
_ledger.append(1)

assert not fnmatch.fnmatch("dbc", "[abc]bc"), "[abc]bc rejects dbc"
_ledger.append(1)

# Negated character class
assert fnmatch.fnmatch("dbc", "[!abc]bc"), "[!abc]bc matches dbc"
_ledger.append(1)

# fnmatchcase is case-sensitive even on case-insensitive platforms
assert fnmatch.fnmatchcase("ABC", "*BC"), "fnmatchcase matches uppercase pattern"
_ledger.append(1)

assert not fnmatch.fnmatchcase("ABC", "*bc"), "fnmatchcase rejects mismatched case"
_ledger.append(1)

# filter() returns only matches
assert fnmatch.filter(["a.py", "b.txt", "c.py", "d.md"], "*.py") == ["a.py", "c.py"], "filter keeps .py only"
_ledger.append(1)

# Empty pattern matches empty string
assert fnmatch.fnmatch("", ""), "empty matches empty"
_ledger.append(1)

# translate returns a regex pattern string
pat = fnmatch.translate("*.py")
assert isinstance(pat, str), "translate returns a string"
_ledger.append(1)

assert len(pat) > 0, "translate returns non-empty pattern"
_ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_fnmatch {sum(_ledger)} asserts")
