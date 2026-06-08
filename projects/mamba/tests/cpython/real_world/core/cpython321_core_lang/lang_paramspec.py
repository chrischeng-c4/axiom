# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_paramspec"
# subject = "cpython321.lang_paramspec"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_paramspec.py"
# status = "filled"
# ///
"""cpython321.lang_paramspec: execute CPython 3.12 seed lang_paramspec"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# lang_paramspec.py — #3349 axis-1 lang ParamSpec + TypeGuard +
# overload AssertionPass seed.
#
# Mamba-authored seed exercising the typing-level surface called out in
# the issue:
#   * ParamSpec — generic over callable signatures
#   * TypeGuard — narrowing return annotation
#   * @overload — multiple typed signatures collapsed to one runtime impl
#
# Contract placement: `spec/` — pins outcome Fail. Mamba runtime gap
# #3496 (ParamSpec rejected by type-checker; @overload triggers codegen
# DuplicateDefinition) blocks AssertionPass today. Once #3496 lands and
# this seed flips to AssertionPass on mamba, drift detection prompts a
# `git mv spec/lang_paramspec.py pass/lang_paramspec.py`.
#
# Surface coverage (asserts run at module scope; no helper closures per
# the mamba top-level def() quirk in test_math.py):
#   1. ParamSpec — declaration; .args and .kwargs attributes; .__name__.
#   2. ParamSpec used in a generic Callable[P, R] decorator factory —
#      the wrapped callable preserves *args/**kwargs.
#   3. TypeGuard — narrowing function returns runtime bool that matches
#      the predicate.
#   4. TypeGuard — negative branch returns False.
#   5. @overload — multiple typed signatures + one runtime implementation;
#      the runtime impl is what executes.
#   6. typing.get_overloads — returns the registered overload signatures.
#
# Contract with `cpython_lib_test_runner.rs`:
#   - Each `assert` runs at top level. Inverting any assert raises
#     AssertionError → non-zero exit → runner classifies as `Fail`.
#   - After every assertion has executed, the seed emits
#     `MAMBA_ASSERTION_PASS: lang_paramspec N asserts` to stdout.

import typing
from typing import (
    Callable,
    ParamSpec,
    TypeGuard,
    TypeVar,
    get_overloads,
    overload,
)

_ledger: list[int] = []

# 1. ParamSpec declaration + introspection.
P = ParamSpec("P")
R = TypeVar("R")

assert isinstance(P, ParamSpec), "ParamSpec('P') returns a ParamSpec instance"
_ledger.append(1)
assert P.__name__ == "P", "ParamSpec.__name__ matches the call argument"
_ledger.append(1)
# .args / .kwargs are attribute access stubs used in signatures.
assert hasattr(P, "args"), "ParamSpec exposes .args"
_ledger.append(1)
assert hasattr(P, "kwargs"), "ParamSpec exposes .kwargs"
_ledger.append(1)


# 2. ParamSpec used in a Callable[P, R] decorator factory.
def _add_logging(fn: Callable[P, R]) -> Callable[P, R]:
    def _wrapped(*args: P.args, **kwargs: P.kwargs) -> R:
        return fn(*args, **kwargs)
    return _wrapped


@_add_logging
def _multiply(x: int, y: int) -> int:
    return x * y


# Decorator-wrapped callable still computes correctly through *args.
assert _multiply(3, 4) - 12 == 0, (
    "ParamSpec-typed decorator preserves positional args (boxed-dodge)"
)
_ledger.append(1)
assert _multiply(x=2, y=5) - 10 == 0, (
    "ParamSpec-typed decorator preserves keyword args (boxed-dodge)"
)
_ledger.append(1)


# 3. TypeGuard — narrowing predicate function.
def _is_int_list(val: object) -> TypeGuard[list[int]]:
    return (
        isinstance(val, list)
        and all(isinstance(x, int) for x in val)
    )


# Positive branch: predicate True for list of ints.
_lst_int: object = [1, 2, 3]
assert _is_int_list(_lst_int) == True, "TypeGuard returns True for list[int]"
_ledger.append(1)
# Predicate is a regular runtime function — narrowing happens at type-check
# time; at runtime it still returns the bool.
assert _is_int_list([]) == True, (
    "TypeGuard returns True for empty list (vacuously list[int])"
)
_ledger.append(1)


# 4. TypeGuard — negative cases.
assert _is_int_list("hello") == False, (
    "TypeGuard returns False for non-list"
)
_ledger.append(1)
assert _is_int_list([1, "two", 3]) == False, (
    "TypeGuard returns False for mixed-type list"
)
_ledger.append(1)
assert _is_int_list(None) == False, "TypeGuard returns False for None"
_ledger.append(1)


# 5. @overload — multiple typed signatures + one runtime impl.
@overload
def _double(x: int) -> int: ...
@overload
def _double(x: str) -> str: ...
def _double(x):  # type: ignore[no-redef]
    return x * 2


# The runtime impl is what executes regardless of which @overload signature
# the type-checker selected.
assert _double(7) - 14 == 0, (
    "@overload runtime impl returns 14 for int input (boxed-dodge)"
)
_ledger.append(1)
assert _double("ab") == "abab", "@overload runtime impl returns 'abab' for str"
_ledger.append(1)


# 6. typing.get_overloads — returns the registered overload signatures.
_overloads = get_overloads(_double)
assert isinstance(_overloads, list), "get_overloads returns a list"
_ledger.append(1)
assert len(_overloads) - 2 == 0, (
    "get_overloads returns 2 entries for two @overload decls (boxed-dodge)"
)
_ledger.append(1)
# Each entry is a callable (the @overload-decorated stub).
assert callable(_overloads[0]), "@overload entry [0] is callable"
_ledger.append(1)
assert callable(_overloads[1]), "@overload entry [1] is callable"
_ledger.append(1)


# Extra coverage: typing module surface.
assert typing.__name__ == "typing", "typing.__name__"
_ledger.append(1)
assert hasattr(typing, "ParamSpec"), "typing exposes ParamSpec"
_ledger.append(1)
assert hasattr(typing, "TypeGuard"), "typing exposes TypeGuard"
_ledger.append(1)
assert hasattr(typing, "overload"), "typing exposes overload"
_ledger.append(1)
assert hasattr(typing, "get_overloads"), "typing exposes get_overloads"
_ledger.append(1)

# Emit the proof-of-execution marker. Per `ASSERTION_PASS_MARKERS` in
# `cpython_lib_test_runner.rs`, presence of this token escalates the
# outcome from ImportPass to AssertionPass.
print(f"MAMBA_ASSERTION_PASS: lang_paramspec {len(_ledger)} asserts")
