# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_format_ops"
# subject = "cpython321.test_format_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_format_ops.py"
# status = "filled"
# ///
"""cpython321.test_format_ops: execute CPython 3.12 seed test_format_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for str format methods.
# Surface: positional/keyword .format(), alignment fill specifiers,
# zero-padding, float precision, center/zfill/rjust/ljust, replace
# count limit, swapcase, capitalize.
# Companion to stub/test_format.py — vendored unittest seed.
_ledger: list[int] = []
assert "{}-{}".format("a", "b") == "a-b"; _ledger.append(1)
assert "{name}={val}".format(name="x", val=42) == "x=42"; _ledger.append(1)
assert "{:>5}".format("ab") == "   ab"; _ledger.append(1)
assert "{:0>3}".format(7) == "007"; _ledger.append(1)
assert "{:5.2f}".format(3.14159) == " 3.14"; _ledger.append(1)
assert "hello".center(11, "*") == "***hello***"; _ledger.append(1)
assert "7".zfill(3) == "007"; _ledger.append(1)
assert "abc".rjust(5) == "  abc"; _ledger.append(1)
assert "abc".ljust(5, "-") == "abc--"; _ledger.append(1)
assert "a-b-c".replace("-", "/", 1) == "a/b-c"; _ledger.append(1)
assert "Hello World".swapcase() == "hELLO wORLD"; _ledger.append(1)
assert "hello".capitalize() == "Hello"; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: test_format_ops {sum(_ledger)} asserts")
