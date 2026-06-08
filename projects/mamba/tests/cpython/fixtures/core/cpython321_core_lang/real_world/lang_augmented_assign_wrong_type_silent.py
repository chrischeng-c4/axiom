# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_augmented_assign_wrong_type_silent"
# subject = "cpython321.lang_augmented_assign_wrong_type_silent"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_augmented_assign_wrong_type_silent.py"
# status = "filled"
# ///
"""cpython321.lang_augmented_assign_wrong_type_silent: execute CPython 3.12 seed lang_augmented_assign_wrong_type_silent"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Spec seed for CPython TypeError contract on augmented-assignment
# (`**=`, `//=`, `|=`, `&=`, `^=`, `<<=`, `>>=`, `-=`, `/=`) when
# the in-place operand is the wrong type. Surface: CPython rejects
# `x **= "abc"` / `x //= "abc"` / `x |= "abc"` / `x &= "abc"` /
# `x ^= "abc"` / `x <<= "abc"` / `x >>= "abc"` for `x: int` with
# TypeError because the in-place dunder (`__ipow__`, `__ifloordiv__`,
# `__ior__`, `__iand__`, `__ixor__`, `__ilshift__`, `__irshift__`)
# falls back to the binary dunder (`__pow__`, `__floordiv__`, etc.)
# which itself rejects the mismatched RHS. Same story for
# `str -= str` / `str /= str` / `str **= str` / `list -= list` /
# `list **= int` / `list /= int` / `list |= list` / `dict -= dict` /
# `tuple += int` — every wrong-type LHS/RHS pair raises TypeError.
# Mamba accepts every form and silently rewrites the target to
# `None` (or leaves the int unchanged for bit-shift), so code like
# `total **= scale_factor` where `scale_factor` is accidentally a
# str silently flips `total` to `None` and every downstream
# arithmetic op surfaces a `unsupported operand` error several
# call-frames away from the actual mistake. Augmented assign is
# distinct from the binary-op divergence covered in
# `lang_typeerror_arithmetic.py` because the failure mutates the
# binding (in-place semantics) instead of producing an expression
# value, and the failure surface routes through the in-place dunder
# dispatch table, not the binary-op table.
#
# Probes (every form CPython rejects, mamba silently mutates):
#   • int **= str        → mamba: None       (TypeError)
#   • int //= str        → mamba: None       (TypeError)
#   • int |= str         → mamba: None       (TypeError)
#   • int &= str         → mamba: None       (TypeError)
#   • int ^= str         → mamba: None       (TypeError)
#   • int <<= str        → mamba: int (unch) (TypeError)
#   • int >>= str        → mamba: int (unch) (TypeError)
#   • str -= str         → mamba: None       (TypeError)
#   • str /= str         → mamba: None       (TypeError)
#   • str **= str        → mamba: None       (TypeError)
#   • list -= list       → mamba: None       (TypeError)
#   • list **= int       → mamba: None       (TypeError)
#   • list /= int        → mamba: None       (TypeError)
#   • list |= list       → mamba: None       (TypeError)
#   • dict -= dict       → mamba: None       (TypeError)
#   • tuple += int       → mamba: None       (TypeError)
#
# CPython contract (uniform across every in-place operator):
#   int **= str / //= / |= / &= / ^= / <<= / >>=
#       → TypeError("unsupported operand type(s) for <op>: 'int' and 'str'");
#   str -= str / /= / **=
#       → TypeError("unsupported operand type(s) for <op>: 'str' and 'str'");
#   list -= list / **= int / /= int / |= list
#       → TypeError("unsupported operand type(s) for <op>: 'list' and …");
#   dict -= dict
#       → TypeError("unsupported operand type(s) for -=: 'dict' and 'dict'");
#   tuple += int
#       → TypeError("can only concatenate tuple (not 'int') to tuple").
#
# `Any`-typed holders push the operand past static type-checkers
# (Pyright) and past mamba's compile-time argtype enforcement so
# the runtime divergence is what's exercised.
from typing import Any
_ledger: list[int] = []

_s: Any = "abc"
_lst: Any = [1, 2, 3]
_d: Any = {"a": 1}

# int **= str
try:
    _x: Any = 5
    _x **= _s
    raise AssertionError("int **= str must raise TypeError")
except TypeError:
    _ledger.append(1)

# int //= str
try:
    _x: Any = 5
    _x //= _s
    raise AssertionError("int //= str must raise TypeError")
except TypeError:
    _ledger.append(1)

# int |= str
try:
    _x: Any = 5
    _x |= _s
    raise AssertionError("int |= str must raise TypeError")
except TypeError:
    _ledger.append(1)

# int &= str
try:
    _x: Any = 5
    _x &= _s
    raise AssertionError("int &= str must raise TypeError")
except TypeError:
    _ledger.append(1)

# int ^= str
try:
    _x: Any = 5
    _x ^= _s
    raise AssertionError("int ^= str must raise TypeError")
except TypeError:
    _ledger.append(1)

# int <<= str
try:
    _x: Any = 5
    _x <<= _s
    raise AssertionError("int <<= str must raise TypeError")
except TypeError:
    _ledger.append(1)

# int >>= str
try:
    _x: Any = 5
    _x >>= _s
    raise AssertionError("int >>= str must raise TypeError")
except TypeError:
    _ledger.append(1)

# str -= str
try:
    _x: Any = "hi"
    _x -= _s
    raise AssertionError("str -= str must raise TypeError")
except TypeError:
    _ledger.append(1)

# str /= str
try:
    _x: Any = "hi"
    _x /= _s
    raise AssertionError("str /= str must raise TypeError")
except TypeError:
    _ledger.append(1)

# str **= str
try:
    _x: Any = "hi"
    _x **= _s
    raise AssertionError("str **= str must raise TypeError")
except TypeError:
    _ledger.append(1)

# list -= list
try:
    _x: Any = [1, 2]
    _x -= _lst
    raise AssertionError("list -= list must raise TypeError")
except TypeError:
    _ledger.append(1)

# list **= int
try:
    _x: Any = [1, 2]
    _x **= 2
    raise AssertionError("list **= int must raise TypeError")
except TypeError:
    _ledger.append(1)

# list /= int
try:
    _x: Any = [1, 2]
    _x /= 2
    raise AssertionError("list /= int must raise TypeError")
except TypeError:
    _ledger.append(1)

# list |= list
try:
    _x: Any = [1, 2]
    _x |= _lst
    raise AssertionError("list |= list must raise TypeError")
except TypeError:
    _ledger.append(1)

# dict -= dict
try:
    _x: Any = {"a": 1}
    _x -= _d
    raise AssertionError("dict -= dict must raise TypeError")
except TypeError:
    _ledger.append(1)

# tuple += int
try:
    _x: Any = (1, 2)
    _x += 3
    raise AssertionError("tuple += int must raise TypeError")
except TypeError:
    _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_augmented_assign_wrong_type_silent {sum(_ledger)} asserts")
