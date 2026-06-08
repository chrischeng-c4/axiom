# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_contextvars_ops"
# subject = "cpython321.test_contextvars_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_contextvars_ops.py"
# status = "filled"
# ///
"""cpython321.test_contextvars_ops: execute CPython 3.12 seed test_contextvars_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for PEP 567 — `contextvars.ContextVar`.
# Surface: ContextVar default, get / set / reset with the token returned
# from set(). Reset restores the prior value, including the default.
# Companion to stub/test_contextvars.py — vendored unittest seed.
import contextvars
_ledger: list[int] = []
v = contextvars.ContextVar("v", default=10)
# Default observed before any set
assert v.get() == 10; _ledger.append(1)
# set() updates the value and returns a reset token
tok = v.set(20)
assert v.get() == 20; _ledger.append(1)
# reset(token) restores the prior value
v.reset(tok)
assert v.get() == 10; _ledger.append(1)
# Multiple set/reset cycles compose
tok2 = v.set(99)
assert v.get() == 99; _ledger.append(1)
tok3 = v.set(100)
assert v.get() == 100; _ledger.append(1)
v.reset(tok3)
assert v.get() == 99; _ledger.append(1)
v.reset(tok2)
assert v.get() == 10; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: test_contextvars_ops {sum(_ledger)} asserts")
