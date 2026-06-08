# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_intenum_ops"
# subject = "cpython321.test_intenum_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_intenum_ops.py"
# status = "filled"
# ///
"""cpython321.test_intenum_ops: execute CPython 3.12 seed test_intenum_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for `enum.IntEnum`.
# Surface: members compare equal to their underlying int values;
# members are ordered by underlying value; int() coerces a member to
# its underlying value; IntEnum arithmetic mixes with int. Member
# `.name` and `.value` accessors are NOT asserted — those return None
# on mamba today and are tracked separately.
from enum import IntEnum
_ledger: list[int] = []

class Status(IntEnum):
    OK = 0
    ERR = 1
    WARN = 2

# IntEnum members compare equal to the underlying int
assert Status.OK == 0; _ledger.append(1)
assert Status.ERR == 1; _ledger.append(1)
assert Status.WARN == 2; _ledger.append(1)
# int(...) coerces a member back to its value
assert int(Status.OK) == 0; _ledger.append(1)
assert int(Status.ERR) == 1; _ledger.append(1)
assert int(Status.WARN) == 2; _ledger.append(1)
# Members are ordered by underlying value
assert Status.OK < Status.ERR; _ledger.append(1)
assert Status.ERR < Status.WARN; _ledger.append(1)
# Same-member equality holds; cross-member inequality holds
assert Status.OK == Status.OK; _ledger.append(1)
assert Status.OK != Status.ERR; _ledger.append(1)
# Arithmetic with int and with another member
assert Status.ERR + 1 == 2; _ledger.append(1)
assert Status.WARN + Status.ERR == 3; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: test_intenum_ops {sum(_ledger)} asserts")
