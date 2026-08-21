# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_math_cmath_maketrans_difflib_value_ops"
# subject = "cpython321.test_math_cmath_maketrans_difflib_value_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_math_cmath_maketrans_difflib_value_ops.py"
# status = "filled"
# ///
"""cpython321.test_math_cmath_maketrans_difflib_value_ops: execute CPython 3.12 seed test_math_cmath_maketrans_difflib_value_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for the value contract of four
# bootstrap stdlib / language surfaces used by every numeric /
# complex-numeric / string-translation / fuzzy-matching path:
# `math` (the documented `gcd` / `lcm` / `log` / `log10` /
# `log2` / `exp` / `sqrt` / `trunc` / `copysign` / `hypot` /
# `isinf` / `isnan` / `floor` / `ceil` / `fabs` / `fmod` /
# `factorial` / `comb` / `perm` / `inf` / `nan` / `pi` / `e` /
# `tau` deeper numeric helper surface), `cmath` (the documented
# `sqrt` / `phase` / `polar` / `rect` / `pi` / `e` complex-
# numeric surface), `str.maketrans` / `str.translate` (the
# documented table-form / dict-form / delete-chars translation
# contracts), and `difflib` (the documented `get_close_matches`
# fuzzy-match helper surface).
#
# The matching subset between mamba and CPython is the math
# deeper helper layer + module hasattr surface for `inf` / `nan`
# / `tau`, the cmath full layer + module hasattr surface, the
# str.maketrans + translate full surface (table form +
# dict form + delete-chars form), and the difflib
# get_close_matches layer + module hasattr surface for
# `SequenceMatcher` / `get_close_matches` / `unified_diff`.
#
# Surface in this fixture:
#   • math — gcd / lcm / log / log10 / log2 / exp / sqrt /
#     trunc / copysign / hypot / isinf / isnan / floor / ceil /
#     fabs / fmod / factorial / comb / perm + inf / nan / pi /
#     e / tau constant surface;
#   • cmath — sqrt / phase / polar / rect / pi constant +
#     module hasattr surface for sqrt / exp / log / phase /
#     polar / rect / pi / e;
#   • str.maketrans + translate — table-form / dict-form /
#     delete-chars-form translation;
#   • difflib — get_close_matches + module hasattr surface
#     for SequenceMatcher / get_close_matches / unified_diff.
#
# Behavioral edges that DIVERGE on mamba (difflib.SequenceMatcher
# constructor returns a bare float instead of a SequenceMatcher
# instance — `sm.ratio()` AttributeError 'float' object has no
# attribute 'ratio', difflib.ndiff / Differ / context_diff
# hasattr False, difflib.unified_diff returns an empty list on
# inputs that should produce a unified-diff output) are covered
# in the matching spec fixture
# `lang_difflib_sequencematcher_silent`.
import math
import cmath
import difflib


_ledger: list[int] = []

# 1) math — integer-arithmetic helpers
assert math.gcd(12, 18) == 6; _ledger.append(1)
assert math.gcd(0, 5) == 5; _ledger.append(1)
assert math.lcm(4, 6) == 12; _ledger.append(1)
assert math.factorial(5) == 120; _ledger.append(1)
assert math.comb(5, 2) == 10; _ledger.append(1)
assert math.perm(5, 2) == 20; _ledger.append(1)

# 2) math — log / exp / sqrt
assert math.log(math.e) == 1.0; _ledger.append(1)
assert math.log10(100) == 2.0; _ledger.append(1)
assert math.log2(8) == 3.0; _ledger.append(1)
assert math.exp(0) == 1.0; _ledger.append(1)
assert math.sqrt(16) == 4.0; _ledger.append(1)

# 3) math — float-rounding helpers
assert math.trunc(3.7) == 3; _ledger.append(1)
assert math.trunc(-3.7) == -3; _ledger.append(1)
assert math.floor(3.7) == 3; _ledger.append(1)
assert math.ceil(3.2) == 4; _ledger.append(1)
assert math.fabs(-3.5) == 3.5; _ledger.append(1)
assert math.fmod(10, 3) == 1.0; _ledger.append(1)
assert math.copysign(3, -1) == -3.0; _ledger.append(1)
assert math.hypot(3, 4) == 5.0; _ledger.append(1)

# 4) math — special-value predicates
assert math.isinf(float("inf")) == True; _ledger.append(1)
assert math.isinf(1) == False; _ledger.append(1)
assert math.isnan(float("nan")) == True; _ledger.append(1)
assert math.isnan(1) == False; _ledger.append(1)

# 5) math — constant surface
assert math.pi == 3.141592653589793; _ledger.append(1)
assert math.e == 2.718281828459045; _ledger.append(1)
assert math.tau == 6.283185307179586; _ledger.append(1)
assert hasattr(math, "inf") == True; _ledger.append(1)
assert hasattr(math, "nan") == True; _ledger.append(1)
assert hasattr(math, "tau") == True; _ledger.append(1)

# 6) cmath — module hasattr surface
assert hasattr(cmath, "sqrt") == True; _ledger.append(1)
assert hasattr(cmath, "exp") == True; _ledger.append(1)
assert hasattr(cmath, "log") == True; _ledger.append(1)
assert hasattr(cmath, "phase") == True; _ledger.append(1)
assert hasattr(cmath, "polar") == True; _ledger.append(1)
assert hasattr(cmath, "rect") == True; _ledger.append(1)
assert hasattr(cmath, "pi") == True; _ledger.append(1)
assert hasattr(cmath, "e") == True; _ledger.append(1)

# 7) cmath — complex-numeric surface
assert cmath.sqrt(-1) == 1j; _ledger.append(1)
assert cmath.sqrt(4) == (2 + 0j); _ledger.append(1)
assert cmath.phase(1j) == 1.5707963267948966; _ledger.append(1)
assert cmath.polar(1 + 1j) == (1.4142135623730951, 0.7853981633974483); _ledger.append(1)
assert cmath.rect(1, 0) == (1 + 0j); _ledger.append(1)
assert cmath.pi == 3.141592653589793; _ledger.append(1)

# 8) str.maketrans / translate — table form
_tr = str.maketrans("abc", "xyz")
assert "abcdef".translate(_tr) == "xyzdef"; _ledger.append(1)

# 9) str.maketrans / translate — dict form
_tr2 = str.maketrans({"a": "1", "b": "2"})
assert "abc".translate(_tr2) == "12c"; _ledger.append(1)

# 10) str.maketrans / translate — delete-chars form
_tr3 = str.maketrans("", "", "aeiou")
assert "hello world".translate(_tr3) == "hll wrld"; _ledger.append(1)

# 11) difflib — get_close_matches fuzzy-match helper
assert difflib.get_close_matches("hello", ["hellow", "hell", "world", "helloo"]) == ["hellow", "helloo", "hell"]; _ledger.append(1)
assert difflib.get_close_matches("aaa", ["bbb", "ccc"]) == []; _ledger.append(1)

# 12) difflib — module hasattr surface
assert hasattr(difflib, "SequenceMatcher") == True; _ledger.append(1)
assert hasattr(difflib, "get_close_matches") == True; _ledger.append(1)
assert hasattr(difflib, "unified_diff") == True; _ledger.append(1)

# NB: difflib.SequenceMatcher constructor returns a bare float
# instead of an instance, difflib.ndiff / Differ / context_diff
# hasattr False, difflib.unified_diff returns an empty list —
# all DIVERGE on mamba — moved to the divergence-spec fixture.

print(f"MAMBA_ASSERTION_PASS: test_math_cmath_maketrans_difflib_value_ops {sum(_ledger)} asserts")
