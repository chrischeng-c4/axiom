# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_sys_platform_version_subscript_silent"
# subject = "cpython321.lang_sys_platform_version_subscript_silent"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_sys_platform_version_subscript_silent.py"
# status = "filled"
# ///
"""cpython321.lang_sys_platform_version_subscript_silent: execute CPython 3.12 seed lang_sys_platform_version_subscript_silent"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Operational AssertionPass seed for SILENT divergences in `sys`
# (exact-value introspection + namedtuple identity + subscript form),
# `setrecursionlimit`, lambda / built-in-type `.__name__`, and the
# `fractions.Fraction` / `decimal.Decimal` value-class types. The
# matching subset (shape/type invariants of `sys` + def-function / user
# class `.__name__`) is covered by
# `test_sys_introspection_function_metadata_ops`; this fixture pins
# the CPython-only contracts that mamba currently elides.
#
# Surface (CPython AssertionPass; mamba diverges silently):
#   • sys.platform — exact value is "darwin" on macOS (mamba: "macos");
#   • sys.maxsize — exact value is (1 << 63) - 1 on a 64-bit build
#     (mamba: ≈ (1 << 47) - 1, a different bit width);
#   • sys.version_info[i] — subscript form works (mamba: KeyError);
#   • sys.int_info / sys.hash_info — namedtuple instances with their
#     own type name (mamba: plain dict);
#   • sys.setrecursionlimit(N) — actually updates the recursion limit
#     (mamba: no-op, getrecursionlimit() stays at the previous value);
#   • (lambda x: x).__name__ — "<lambda>" (mamba: None);
#   • int.__name__ / str.__name__ / list.__name__ / dict.__name__ /
#     tuple.__name__ / bool.__name__ / float.__name__ — the built-in
#     type names as strings (mamba: returns an unbound-method handle
#     instead of the type name);
#   • sys.maxunicode — typically 0x10FFFF (mamba: None);
#   • fractions.Fraction(1, 2) — a Fraction instance (mamba: returns
#     an int-typed 0 / float, no Fraction class);
#   • decimal.Decimal("1.5") — a Decimal instance (mamba: returns an
#     int-typed 0 / float, no Decimal class).
import sys
from typing import Any

_ledger: list[int] = []

# 1) sys.platform exact value — "darwin" on macOS
assert sys.platform == "darwin"; _ledger.append(1)
assert sys.platform != "macos"; _ledger.append(1)

# 2) sys.maxsize exact value — 64-bit signed max
assert sys.maxsize == (1 << 63) - 1; _ledger.append(1)
assert sys.maxsize == 9223372036854775807; _ledger.append(1)
assert sys.maxsize > (1 << 47); _ledger.append(1)

# 3) sys.version_info[i] subscript form
_v0: Any = sys.version_info[0]
assert _v0 == 3; _ledger.append(1)
_v1: Any = sys.version_info[1]
assert isinstance(_v1, int); _ledger.append(1)
assert _v1 >= 0; _ledger.append(1)

# 4) sys.int_info / sys.hash_info — namedtuple-like (not plain dict)
assert type(sys.int_info).__name__ == "int_info"; _ledger.append(1)
assert type(sys.hash_info).__name__ == "hash_info"; _ledger.append(1)
assert not isinstance(sys.int_info, dict); _ledger.append(1)
assert not isinstance(sys.hash_info, dict); _ledger.append(1)

# 5) sys.setrecursionlimit actually updates the limit
_old_rl: Any = sys.getrecursionlimit()
sys.setrecursionlimit(2000)
_new_rl: Any = sys.getrecursionlimit()
assert _new_rl == 2000; _ledger.append(1)
assert _new_rl != _old_rl; _ledger.append(1)
sys.setrecursionlimit(_old_rl)
_restored_rl: Any = sys.getrecursionlimit()
assert _restored_rl == _old_rl; _ledger.append(1)

# 6) lambda.__name__ == "<lambda>"
_lam: Any = lambda x: x
assert _lam.__name__ == "<lambda>"; _ledger.append(1)
assert isinstance(_lam.__name__, str); _ledger.append(1)

# 7) Built-in type .__name__ is the type-name string
assert int.__name__ == "int"; _ledger.append(1)
assert str.__name__ == "str"; _ledger.append(1)
assert list.__name__ == "list"; _ledger.append(1)
assert dict.__name__ == "dict"; _ledger.append(1)
assert tuple.__name__ == "tuple"; _ledger.append(1)
assert bool.__name__ == "bool"; _ledger.append(1)
assert float.__name__ == "float"; _ledger.append(1)
assert isinstance(int.__name__, str); _ledger.append(1)
assert isinstance(float.__name__, str); _ledger.append(1)

# 8) sys.maxunicode — typically 0x10FFFF (1114111)
assert isinstance(sys.maxunicode, int); _ledger.append(1)
assert sys.maxunicode == 0x10FFFF; _ledger.append(1)
assert sys.maxunicode == 1114111; _ledger.append(1)

# 9) fractions.Fraction(num, den) returns a Fraction instance
from fractions import Fraction
_f: Any = Fraction(1, 2)
assert type(_f).__name__ == "Fraction"; _ledger.append(1)
assert isinstance(_f, Fraction); _ledger.append(1)
assert _f == Fraction(1, 2); _ledger.append(1)
_f2: Any = Fraction(3, 4)
assert _f2 == Fraction(3, 4); _ledger.append(1)
# Fraction arithmetic returns Fraction
_f_sum: Any = Fraction(1, 2) + Fraction(1, 3)
assert _f_sum == Fraction(5, 6); _ledger.append(1)
assert type(_f_sum).__name__ == "Fraction"; _ledger.append(1)

# 10) decimal.Decimal(str) returns a Decimal instance
from decimal import Decimal
_d: Any = Decimal("1.5")
assert type(_d).__name__ == "Decimal"; _ledger.append(1)
assert isinstance(_d, Decimal); _ledger.append(1)
assert _d == Decimal("1.5"); _ledger.append(1)
# Decimal arithmetic returns Decimal
_d_sum: Any = Decimal("1.5") + Decimal("2.5")
assert _d_sum == Decimal("4.0"); _ledger.append(1)
assert type(_d_sum).__name__ == "Decimal"; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_sys_platform_version_subscript_silent {sum(_ledger)} asserts")
