# Operational AssertionPass divergence-spec fixture for the
# silent value-contract divergence of the
# `math` float-returning ops / `array.array` iteration /
# `argparse` deep surface / `unittest` deep surface /
# `dataclasses` instance value contract / `sched.scheduler` /
# `platform.uname_result` seven-pack pinned to atomic 240:
# `math.copysign / math.log / math.log2 / math.log10 / math.exp
# / math.sqrt / math.pow / math.fmod / math.ldexp / math.dist /
# math.hypot / math.pi / math.e / math.tau / math.inf` (the
# documented "float-returning op returns a Python `float`"
# value contract — mamba silently returns a boxed-handle
# integer whose `type(...).__name__` resolves to `int` instead
# of `float`, so downstream float comparisons and `isinstance`
# checks break), `list(array.array("i", [1, 2, 3]))` (the
# documented "array.array is iterable and yields its elements"
# value contract — mamba's array iteration silently collapses
# to the empty list and append/extend/pop produce no observable
# elements), `argparse.Namespace / Action / FileType /
# HelpFormatter / ArgumentTypeError / ArgumentError / SUPPRESS
# / REMAINDER / OPTIONAL / ZERO_OR_MORE / ONE_OR_MORE / PARSER`
# (the documented deep surface — mamba's `argparse` module
# dict only exposes `ArgumentParser` and silently drops the
# rest), `unittest.TestSuite / TestLoader / TextTestRunner /
# TextTestResult / defaultTestLoader / findTestCases /
# SkipTest` (the documented deep surface — mamba's `unittest`
# module dict only exposes the partial set covered in the
# pass fixture and silently drops the rest), the
# `@dataclasses.dataclass`-decorated instance value contract
# (the documented "ctor binds fields, asdict/astuple/fields
# reflect them, str repr renders `ClassName(x=1, y=2)`" —
# mamba binds no fields and renders `<_Point instance>`),
# `sched.scheduler` (the documented top-level class — mamba
# does not expose it), and `platform.uname_result` (the
# documented top-level named-tuple result class — mamba does
# not expose it).
#
# Behavioral edges that CONFORM on mamba (math integer-/bool-
# returning ops factorial/gcd/lcm/isclose/isnan/isinf/floor/
# ceil/trunc/comb/perm/frexp/modf + math.nan type contract +
# math 24-name hasattr surface; array.array class binding +
# typecodes hasattr + typecode/itemsize attr; argparse
# ArgumentParser binding; unittest TestCase/main/skip/skipIf/
# skipUnless/expectedFailure partial surface; doctest 11-name
# full surface; functools 13-name deeper surface + partial
# call + reduce sum + reduce with init) are covered in the
# matching pass fixture
# `test_math_functools_unittest_doctest_argparse_value_ops`.
from typing import Any
import math as _math_mod
import array as _array_mod
import argparse as _argparse_mod
import unittest as _unittest_mod
import dataclasses as _dataclasses_mod
import sched as _sched_mod
import platform as _platform_mod

math_mod: Any = _math_mod
array_mod: Any = _array_mod
argparse_mod: Any = _argparse_mod
unittest_mod: Any = _unittest_mod
dataclasses_mod: Any = _dataclasses_mod
sched_mod: Any = _sched_mod
platform_mod: Any = _platform_mod


@_dataclasses_mod.dataclass
class _Point:
    x: int
    y: int


_ledger: list[int] = []

# 1) math float-returning ops — type contract
#    (mamba: returns boxed-handle int instead of float)
assert type(math_mod.pi).__name__ == "float"; _ledger.append(1)
assert type(math_mod.e).__name__ == "float"; _ledger.append(1)
assert type(math_mod.tau).__name__ == "float"; _ledger.append(1)
assert type(math_mod.inf).__name__ == "float"; _ledger.append(1)
assert type(math_mod.sqrt(16)).__name__ == "float"; _ledger.append(1)
assert type(math_mod.exp(0)).__name__ == "float"; _ledger.append(1)
assert type(math_mod.log(math_mod.e)).__name__ == "float"; _ledger.append(1)
assert type(math_mod.log2(8)).__name__ == "float"; _ledger.append(1)
assert type(math_mod.log10(100)).__name__ == "float"; _ledger.append(1)
assert type(math_mod.pow(2, 10)).__name__ == "float"; _ledger.append(1)
assert type(math_mod.fmod(10, 3)).__name__ == "float"; _ledger.append(1)
assert type(math_mod.ldexp(0.5, 4)).__name__ == "float"; _ledger.append(1)
assert type(math_mod.copysign(1, -1)).__name__ == "float"; _ledger.append(1)
assert type(math_mod.dist((0, 0), (3, 4))).__name__ == "float"; _ledger.append(1)
assert type(math_mod.hypot(3, 4)).__name__ == "float"; _ledger.append(1)

# 2) array.array iteration value contract
#    (mamba: silently collapses to empty list / no observable elements)
assert list(array_mod.array("i", [1, 2, 3])) == [1, 2, 3]; _ledger.append(1)
_a1 = array_mod.array("i", [1, 2, 3])
_a1.append(4)
assert list(_a1) == [1, 2, 3, 4]; _ledger.append(1)
_a2 = array_mod.array("i", [1, 2, 3])
_a2.extend([4, 5])
assert list(_a2) == [1, 2, 3, 4, 5]; _ledger.append(1)
_a3 = array_mod.array("i", [1, 2, 3, 4])
_a3.pop()
assert list(_a3) == [1, 2, 3]; _ledger.append(1)

# 3) argparse deep surface
#    (mamba: missing — only ArgumentParser is exposed)
assert hasattr(argparse_mod, "Namespace") == True; _ledger.append(1)
assert hasattr(argparse_mod, "Action") == True; _ledger.append(1)
assert hasattr(argparse_mod, "FileType") == True; _ledger.append(1)
assert hasattr(argparse_mod, "HelpFormatter") == True; _ledger.append(1)
assert hasattr(argparse_mod, "ArgumentTypeError") == True; _ledger.append(1)
assert hasattr(argparse_mod, "ArgumentError") == True; _ledger.append(1)
assert hasattr(argparse_mod, "SUPPRESS") == True; _ledger.append(1)
assert hasattr(argparse_mod, "REMAINDER") == True; _ledger.append(1)
assert hasattr(argparse_mod, "OPTIONAL") == True; _ledger.append(1)
assert hasattr(argparse_mod, "ZERO_OR_MORE") == True; _ledger.append(1)
assert hasattr(argparse_mod, "ONE_OR_MORE") == True; _ledger.append(1)
assert hasattr(argparse_mod, "PARSER") == True; _ledger.append(1)

# 4) unittest deep surface
#    (mamba: missing — only TestCase/main/skip*/expectedFailure are exposed)
assert hasattr(unittest_mod, "TestSuite") == True; _ledger.append(1)
assert hasattr(unittest_mod, "TestLoader") == True; _ledger.append(1)
assert hasattr(unittest_mod, "TextTestRunner") == True; _ledger.append(1)
assert hasattr(unittest_mod, "TextTestResult") == True; _ledger.append(1)
assert hasattr(unittest_mod, "defaultTestLoader") == True; _ledger.append(1)
assert hasattr(unittest_mod, "findTestCases") == True; _ledger.append(1)
assert hasattr(unittest_mod, "SkipTest") == True; _ledger.append(1)

# 5) @dataclass instance value contract
#    (mamba: binds no fields — ctor returns `<_Point instance>`,
#    `.x` is None, asdict/astuple/fields error)
_pt = _Point(1, 2)
assert _pt.x == 1; _ledger.append(1)
assert _pt.y == 2; _ledger.append(1)
assert dataclasses_mod.asdict(_pt) == {"x": 1, "y": 2}; _ledger.append(1)
assert dataclasses_mod.astuple(_pt) == (1, 2); _ledger.append(1)
assert [f.name for f in dataclasses_mod.fields(_Point)] == ["x", "y"]; _ledger.append(1)
assert _Point(1, 2) == _Point(1, 2); _ledger.append(1)

# 6) sched.scheduler — top-level class
#    (mamba: missing)
assert hasattr(sched_mod, "scheduler") == True; _ledger.append(1)

# 7) platform.uname_result — top-level named-tuple result class
#    (mamba: missing)
assert hasattr(platform_mod, "uname_result") == True; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_math_floats_array_argparse_unittest_dataclass_silent {sum(_ledger)} asserts")
