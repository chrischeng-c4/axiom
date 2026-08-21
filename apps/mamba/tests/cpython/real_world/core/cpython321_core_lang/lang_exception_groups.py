# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_exception_groups"
# subject = "cpython321.lang_exception_groups"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/lang_exception_groups.py"
# status = "filled"
# ///
"""cpython321.lang_exception_groups: execute CPython 3.12 seed lang_exception_groups"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# lang_exception_groups.py — #3339 axis-1 exception groups + except* (PEP 654) seed.
#
# Exercises the core PEP 654 surface mamba services today:
#   1. ExceptionGroup("msg", [exc, exc, ...]) constructor + type identity
#   2. eg.message attribute readback
#   3. eg.exceptions attribute readback (tuple of contained exceptions)
#   4. raise ExceptionGroup → except* ValueError + except* TypeError both fire
#      for their matching subtype in the same try block
#   5. except* on an ExceptionGroup containing a single exception fires once
#   6. except* Exception serves as a fallback when no narrow handler matches
#   7. ExceptionGroup with three different exception types — each branch fires
#      independently
#
# Mamba quirks (tracked separately as #3481):
#   * BaseExceptionGroup name is undefined at type-check time
#   * eg.split() not exercised here
#   * eg.derive() not exercised here
#
# Contract: AssertionError → Fail; MAMBA_ASSERTION_PASS → AssertionPass.

_ledger: list[int] = []

# (1) Bare ExceptionGroup constructor produces an ExceptionGroup-typed object
_eg = ExceptionGroup("msg", [ValueError("v"), TypeError("t")])
assert type(_eg).__name__ == "ExceptionGroup", (
    f"ExceptionGroup(...) is ExceptionGroup-typed, got {type(_eg).__name__!r}"
)
_ledger.append(1)

# (2) .message attribute readback
assert _eg.message == "msg", (
    f"ExceptionGroup.message == 'msg', got {_eg.message!r}"
)
_ledger.append(1)

# (3) .exceptions attribute is a tuple of the contained exceptions
assert len(_eg.exceptions) - 2 == 0, (
    f"ExceptionGroup.exceptions has 2 items, got {len(_eg.exceptions)!r}"
)
_ledger.append(1)

# (4) raise ExceptionGroup → both except* branches fire for matching subtypes
_caught_v = None
_caught_t = None
try:
    raise ExceptionGroup("oops", [ValueError("v"), TypeError("t")])
except* ValueError:
    _caught_v = "got_v"
except* TypeError:
    _caught_t = "got_t"

assert _caught_v == "got_v", (
    f"except* ValueError fires on ExceptionGroup containing ValueError, "
    f"got _caught_v={_caught_v!r}"
)
_ledger.append(1)
assert _caught_t == "got_t", (
    f"except* TypeError fires on ExceptionGroup containing TypeError, "
    f"got _caught_t={_caught_t!r}"
)
_ledger.append(1)

# (5) except* on a single-exception ExceptionGroup fires once
_caught_single = None
try:
    raise ExceptionGroup("g", [ValueError("only")])
except* ValueError:
    _caught_single = "got"

assert _caught_single == "got", (
    f"except* ValueError on single-element group fires, got {_caught_single!r}"
)
_ledger.append(1)

# (6) except* Exception serves as a fallback when narrow handlers miss
_caught_fb = None
try:
    raise ExceptionGroup("g", [ValueError("v")])
except* KeyError:
    _caught_fb = "wrong"
except* Exception:
    _caught_fb = "fallback"

assert _caught_fb == "fallback", (
    f"except* Exception fallback fires when narrow handler misses, "
    f"got {_caught_fb!r}"
)
_ledger.append(1)

# (7) ExceptionGroup with three different exception types — each branch fires
_v = _t = _k = None
try:
    raise ExceptionGroup("triple", [
        ValueError("v"),
        TypeError("t"),
        KeyError("k"),
    ])
except* ValueError:
    _v = "v"
except* TypeError:
    _t = "t"
except* KeyError:
    _k = "k"

assert _v == "v" and _t == "t" and _k == "k", (
    f"except* fires independently for each of three subtype branches, "
    f"got _v={_v!r}, _t={_t!r}, _k={_k!r}"
)
_ledger.append(1)

# (8) ExceptionGroup symbol is exposed at module level (builtins)
assert ExceptionGroup is not None, "ExceptionGroup symbol is exposed"
_ledger.append(1)

# (9) ExceptionGroup is callable as a constructor with the (message, list)
#     signature
_eg2 = ExceptionGroup("two", [ValueError("a"), ValueError("b")])
assert type(_eg2).__name__ == "ExceptionGroup", (
    f"second ExceptionGroup construction also yields an ExceptionGroup, "
    f"got {type(_eg2).__name__!r}"
)
_ledger.append(1)
assert _eg2.message == "two", (
    f"second ExceptionGroup .message == 'two', got {_eg2.message!r}"
)
_ledger.append(1)

# (10) Two ValueErrors in one group still fire the except* ValueError branch
_caught_multi = None
try:
    raise ExceptionGroup("twin", [ValueError("a"), ValueError("b")])
except* ValueError:
    _caught_multi = "got"

assert _caught_multi == "got", (
    f"except* ValueError fires for a group of two ValueErrors, "
    f"got {_caught_multi!r}"
)
_ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_exception_groups {sum(_ledger)} asserts")
