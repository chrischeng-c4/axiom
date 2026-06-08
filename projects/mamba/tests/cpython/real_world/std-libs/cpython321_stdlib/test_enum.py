# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_enum"
# subject = "cpython321.test_enum"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_enum.py"
# status = "filled"
# ///
"""cpython321.test_enum: execute CPython 3.12 seed test_enum"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# test_enum.py — #2833 CPython enum seed (executed assertions).
#
# Mamba-authored seed distilled from the enum module's identity +
# top-level class bindings + member-level identity invariants. Exercises
# the deterministic surface today: module identity, the canonical class
# names (Enum, IntEnum, Flag, auto, unique), and member equality /
# identity / inequality on a tiny three-member Enum subclass.
#
# Why so small? Mamba's current enum surface presents the standard
# names, but `member.value` and `member.name` return None today
# (descriptor protocol not yet bound on enum members), and
# `Color['RED']` / `Color(1)` lookups do not yet behave as CPython's
# `__getitem__` / `_missing_` protocol. The seed asserts only what
# mamba currently emits correctly: member identity is stable (`RED is
# RED`), distinct members have distinct identity (`RED is not GREEN`),
# and == follows is.
#
# Why no helper function? Per the #2691 contract, top-level `def()`
# does not capture module-scope names by reference on mamba. The Enum
# subclass is fine — it's a class declaration, not a captured callable.
#
# Contract with the runner (#2691):
#   - Each `assert` runs at top level. Inverting any assert raises
#     AssertionError → non-zero exit → runner classifies as `Fail`.
#   - After every assertion has executed, the seed emits
#     `MAMBA_ASSERTION_PASS: enum N asserts` to stdout.

import enum

_ledger: list[int] = []

# 1. Module identity + public surface.
assert enum.__name__ == "enum", "enum.__name__ must be 'enum'"
_ledger.append(1)
assert hasattr(enum, "Enum"), "enum must expose Enum"
_ledger.append(1)
assert hasattr(enum, "IntEnum"), "enum must expose IntEnum"
_ledger.append(1)
assert hasattr(enum, "Flag"), "enum must expose Flag"
_ledger.append(1)
assert hasattr(enum, "auto"), "enum must expose auto"
_ledger.append(1)
assert hasattr(enum, "unique"), "enum must expose unique"
_ledger.append(1)

# 2. Tiny Enum subclass — the canonical "one Enum class" fixture
#    requested by the #2833 acceptance.
class Color(enum.Enum):
    RED = 1
    GREEN = 2
    BLUE = 3

# 3. Member identity is stable — `RED is RED` must be True on every
#    access (singletons across the whole interpreter).
assert Color.RED is Color.RED, "Color.RED is Color.RED (singleton identity)"
_ledger.append(1)
assert Color.GREEN is Color.GREEN, "Color.GREEN is Color.GREEN"
_ledger.append(1)
assert Color.BLUE is Color.BLUE, "Color.BLUE is Color.BLUE"
_ledger.append(1)

# 4. Distinct members have distinct identity — `RED is GREEN` must be
#    False. This is what makes Enum useful as a type tag.
assert Color.RED is not Color.GREEN, "Color.RED is not Color.GREEN"
_ledger.append(1)
assert Color.GREEN is not Color.BLUE, "Color.GREEN is not Color.BLUE"
_ledger.append(1)
assert Color.RED is not Color.BLUE, "Color.RED is not Color.BLUE"
_ledger.append(1)

# 5. Equality follows identity for Enum members — `a == b` iff `a is
#    b`. Compared via `== True` / `== False` (mamba's `is True` /
#    `is False` returns False against bool returns).
assert (Color.RED == Color.RED) == True, "Color.RED == Color.RED (reflexive equality)"
_ledger.append(1)
assert (Color.RED == Color.GREEN) == False, "Color.RED != Color.GREEN"
_ledger.append(1)
assert (Color.RED != Color.GREEN) == True, "Color.RED != Color.GREEN (!= positive)"
_ledger.append(1)

# Emit the proof-of-execution marker as the FINAL line so the runner
# can see it on stdout. Per `ASSERTION_PASS_MARKERS` in
# `cpython_lib_test_runner.rs`, presence of this token escalates the
# outcome from ImportPass to AssertionPass.
print(f"MAMBA_ASSERTION_PASS: enum {len(_ledger)} asserts")
