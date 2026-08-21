# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_logging_levels_ops"
# subject = "cpython321.test_logging_levels_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_logging_levels_ops.py"
# status = "filled"
# ///
"""cpython321.test_logging_levels_ops: execute CPython 3.12 seed test_logging_levels_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for `logging` module level constants
# and getLevelName mapping.
# Surface: DEBUG=10, INFO=20, WARNING=30, ERROR=40, CRITICAL=50 with
# strict ordering invariant DEBUG<INFO<WARNING<ERROR<CRITICAL.
# Companion to stub/test_logging.py — vendored unittest seed.
import logging
_ledger: list[int] = []
assert logging.DEBUG == 10; _ledger.append(1)
assert logging.INFO == 20; _ledger.append(1)
assert logging.WARNING == 30; _ledger.append(1)
assert logging.ERROR == 40; _ledger.append(1)
assert logging.CRITICAL == 50; _ledger.append(1)
# Strict ordering — production filters rely on this monotonicity
assert logging.DEBUG < logging.INFO; _ledger.append(1)
assert logging.INFO < logging.WARNING; _ledger.append(1)
assert logging.WARNING < logging.ERROR; _ledger.append(1)
assert logging.ERROR < logging.CRITICAL; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: test_logging_levels_ops {sum(_ledger)} asserts")
