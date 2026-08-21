# Atomic 240 pass conformance — math (integer-returning + class surface) /
# array partial / argparse partial / unittest partial / doctest /
# functools deeper + value ops / platform partial surface that match
# between CPython 3.12 and mamba.
import math
import array
import argparse
import unittest
import doctest
import functools


_ledger: list[int] = []

# 1) math integer-returning value ops + bool returns
assert math.factorial(5) == 120; _ledger.append(1)
assert math.gcd(12, 18) == 6; _ledger.append(1)
assert math.gcd(7, 13) == 1; _ledger.append(1)
assert math.lcm(4, 6) == 12; _ledger.append(1)
assert math.isclose(1.0, 1.0001) == False; _ledger.append(1)
assert math.isnan(float("nan")) == True; _ledger.append(1)
assert math.isinf(float("inf")) == True; _ledger.append(1)
assert math.floor(3.7) == 3; _ledger.append(1)
assert math.ceil(3.2) == 4; _ledger.append(1)
assert math.trunc(3.7) == 3; _ledger.append(1)
assert math.comb(5, 2) == 10; _ledger.append(1)
assert math.perm(5, 2) == 20; _ledger.append(1)
assert math.frexp(8) == (0.5, 4); _ledger.append(1)
assert math.modf(3.5) == (0.5, 3.0); _ledger.append(1)

# 2) math.nan type contract (mamba returns 'float' for nan)
assert type(math.nan).__name__ == "float"; _ledger.append(1)

# 3) math hasattr surface
assert hasattr(math, "cbrt") == True; _ledger.append(1)
assert hasattr(math, "remainder") == True; _ledger.append(1)
assert hasattr(math, "nextafter") == True; _ledger.append(1)
assert hasattr(math, "ulp") == True; _ledger.append(1)
assert hasattr(math, "prod") == True; _ledger.append(1)
assert hasattr(math, "fsum") == True; _ledger.append(1)
assert hasattr(math, "sin") == True; _ledger.append(1)
assert hasattr(math, "cos") == True; _ledger.append(1)
assert hasattr(math, "tan") == True; _ledger.append(1)
assert hasattr(math, "asin") == True; _ledger.append(1)
assert hasattr(math, "acos") == True; _ledger.append(1)
assert hasattr(math, "atan") == True; _ledger.append(1)
assert hasattr(math, "atan2") == True; _ledger.append(1)
assert hasattr(math, "sinh") == True; _ledger.append(1)
assert hasattr(math, "cosh") == True; _ledger.append(1)
assert hasattr(math, "tanh") == True; _ledger.append(1)
assert hasattr(math, "degrees") == True; _ledger.append(1)
assert hasattr(math, "radians") == True; _ledger.append(1)
assert hasattr(math, "pi") == True; _ledger.append(1)
assert hasattr(math, "e") == True; _ledger.append(1)
assert hasattr(math, "tau") == True; _ledger.append(1)
assert hasattr(math, "inf") == True; _ledger.append(1)
assert hasattr(math, "nan") == True; _ledger.append(1)

# 4) array partial — class binding + typecode/itemsize
assert hasattr(array, "array") == True; _ledger.append(1)
assert hasattr(array, "typecodes") == True; _ledger.append(1)
assert array.array("i", [1, 2, 3]).typecode == "i"; _ledger.append(1)
assert array.array("i", [1, 2, 3]).itemsize == 4; _ledger.append(1)
assert array.array("d", [1.0]).typecode == "d"; _ledger.append(1)
assert array.array("d", [1.0]).itemsize == 8; _ledger.append(1)

# 5) argparse partial — ArgumentParser only
assert hasattr(argparse, "ArgumentParser") == True; _ledger.append(1)

# 6) unittest partial surface
assert hasattr(unittest, "TestCase") == True; _ledger.append(1)
assert hasattr(unittest, "main") == True; _ledger.append(1)
assert hasattr(unittest, "skip") == True; _ledger.append(1)
assert hasattr(unittest, "skipIf") == True; _ledger.append(1)
assert hasattr(unittest, "skipUnless") == True; _ledger.append(1)
assert hasattr(unittest, "expectedFailure") == True; _ledger.append(1)

# 7) doctest full surface
assert hasattr(doctest, "testmod") == True; _ledger.append(1)
assert hasattr(doctest, "testfile") == True; _ledger.append(1)
assert hasattr(doctest, "DocTestFinder") == True; _ledger.append(1)
assert hasattr(doctest, "DocTestRunner") == True; _ledger.append(1)
assert hasattr(doctest, "DocTestParser") == True; _ledger.append(1)
assert hasattr(doctest, "Example") == True; _ledger.append(1)
assert hasattr(doctest, "DocTest") == True; _ledger.append(1)
assert hasattr(doctest, "DocTestCase") == True; _ledger.append(1)
assert hasattr(doctest, "ELLIPSIS") == True; _ledger.append(1)
assert hasattr(doctest, "SKIP") == True; _ledger.append(1)
assert hasattr(doctest, "NORMALIZE_WHITESPACE") == True; _ledger.append(1)

# 8) functools deeper surface + value ops
assert hasattr(functools, "partial") == True; _ledger.append(1)
assert hasattr(functools, "partialmethod") == True; _ledger.append(1)
assert hasattr(functools, "cached_property") == True; _ledger.append(1)
assert hasattr(functools, "wraps") == True; _ledger.append(1)
assert hasattr(functools, "update_wrapper") == True; _ledger.append(1)
assert hasattr(functools, "lru_cache") == True; _ledger.append(1)
assert hasattr(functools, "cache") == True; _ledger.append(1)
assert hasattr(functools, "reduce") == True; _ledger.append(1)
assert hasattr(functools, "total_ordering") == True; _ledger.append(1)
assert hasattr(functools, "singledispatch") == True; _ledger.append(1)
assert hasattr(functools, "cmp_to_key") == True; _ledger.append(1)
assert hasattr(functools, "WRAPPER_ASSIGNMENTS") == True; _ledger.append(1)
assert hasattr(functools, "WRAPPER_UPDATES") == True; _ledger.append(1)
_add = functools.partial(lambda a, b: a + b, 5)
assert _add(3) == 8; _ledger.append(1)
assert functools.reduce(lambda a, b: a + b, [1, 2, 3, 4]) == 10; _ledger.append(1)
assert functools.reduce(lambda a, b: a + b, [1, 2, 3], 100) == 106; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_math_functools_unittest_doctest_argparse_value_ops {sum(_ledger)} asserts")
