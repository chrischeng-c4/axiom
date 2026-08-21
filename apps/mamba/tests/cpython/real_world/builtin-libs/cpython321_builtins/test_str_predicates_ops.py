# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "cpython321_builtins"
# dimension = "real_world"
# case = "test_str_predicates_ops"
# subject = "cpython321.test_str_predicates_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_str_predicates_ops.py"
# status = "filled"
# ///
"""cpython321.test_str_predicates_ops: execute CPython 3.12 seed test_str_predicates_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for str classification predicates.
# Surface: isalpha, isalnum, isdigit, isupper, islower, istitle,
# isspace. Each predicate is asserted in both the positive and
# negative direction so a regression that always returns True (or
# always False) cannot silently pass.
_ledger: list[int] = []
# Positive cases
assert "hello".isalpha(); _ledger.append(1)
assert "hello123".isalnum(); _ledger.append(1)
assert "123".isdigit(); _ledger.append(1)
assert "HELLO".isupper(); _ledger.append(1)
assert "hello".islower(); _ledger.append(1)
assert "Hello World".istitle(); _ledger.append(1)
assert "   ".isspace(); _ledger.append(1)
# Negative cases — a regression that always returns True would
# fail these.
assert not "hello123".isalpha(); _ledger.append(1)
assert not "hello world".isalnum(); _ledger.append(1)
assert not "12a".isdigit(); _ledger.append(1)
assert not "Hello".isupper(); _ledger.append(1)
assert not "Hello".islower(); _ledger.append(1)
assert not "Hello world".istitle(); _ledger.append(1)
assert not "abc".isspace(); _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: test_str_predicates_ops {sum(_ledger)} asserts")
