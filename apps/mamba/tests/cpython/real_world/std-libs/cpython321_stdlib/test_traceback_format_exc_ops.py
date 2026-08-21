# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_traceback_format_exc_ops"
# subject = "cpython321.test_traceback_format_exc_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_traceback_format_exc_ops.py"
# status = "filled"
# ///
"""cpython321.test_traceback_format_exc_ops: execute CPython 3.12 seed test_traceback_format_exc_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for `traceback` formatting helpers
# in the post-except handler context. Surface: `traceback.format_exc()`
# returns a string carrying the active exception's type name and
# message; `traceback.format_exception_only(type, value)` returns a
# non-empty list whose joined text contains the same. The type-name
# substring is the canonical class name (`ValueError`, `RuntimeError`,
# `TypeError`, `KeyError`) and the message substring is the original
# argument. Nested raise-from-except keeps the *outer* exception's
# type and message in `format_exc()` when caught one level up.
import traceback
_ledger: list[int] = []

# format_exc returns a str carrying type+message
try:
    raise ValueError("oops")
except ValueError as e:
    s = traceback.format_exc()
assert isinstance(s, str); _ledger.append(1)
assert "ValueError" in s; _ledger.append(1)
assert "oops" in s; _ledger.append(1)

# format_exception_only returns a non-empty list
try:
    raise RuntimeError("bad")
except RuntimeError as e:
    parts = traceback.format_exception_only(type(e), e)
assert isinstance(parts, list); _ledger.append(1)
assert len(parts) > 0; _ledger.append(1)
joined = "".join(parts)
assert "RuntimeError" in joined; _ledger.append(1)
assert "bad" in joined; _ledger.append(1)

# Other exception types reflected in format_exc
try:
    raise TypeError("t1")
except TypeError as e:
    sa = traceback.format_exc()
assert "TypeError" in sa; _ledger.append(1)

try:
    raise KeyError("k")
except KeyError as e:
    sb = traceback.format_exc()
assert "KeyError" in sb; _ledger.append(1)

# Nested raise: outer exception's type+message visible
try:
    try:
        raise ValueError("inner")
    except ValueError:
        raise RuntimeError("outer")
except RuntimeError as e:
    sc = traceback.format_exc()
assert "RuntimeError" in sc; _ledger.append(1)
assert "outer" in sc; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_traceback_format_exc_ops {sum(_ledger)} asserts")
