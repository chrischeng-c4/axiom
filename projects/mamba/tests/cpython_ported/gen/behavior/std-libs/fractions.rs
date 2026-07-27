use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/fractions/construct_from_decimal.py`.
#[test]
fn test_gen_behavior_std_libs_fractions_construct_from_decimal() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fractions"
# dimension = "behavior"
# case = "construct_from_decimal"
# subject = "fractions.Fraction"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_fractions.py"
# status = "filled"
# ///
"""fractions.Fraction: Fraction(Decimal('1.5')) constructs the exact rational 3/2 from a decimal.Decimal"""
from decimal import Decimal
from fractions import Fraction

assert Fraction(Decimal("1.5")) == Fraction(3, 2), "Decimal 1.5 -> 3/2"
assert Fraction(Decimal("0.25")) == Fraction(1, 4), "Decimal 0.25 -> 1/4"

print("construct_from_decimal OK")
"###);
    assert_output(&out, r###"construct_from_decimal OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/fractions/construct_from_string.py`.
#[test]
fn test_gen_behavior_std_libs_fractions_construct_from_string() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fractions"
# dimension = "behavior"
# case = "construct_from_string"
# subject = "fractions.Fraction"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_fractions.py"
# status = "filled"
# ///
"""fractions.Fraction: Fraction parses fraction and decimal strings: '3/4' -> 3/4, '0.5' -> 1/2, '-3/4' -> -3/4, with surrounding whitespace stripped"""
from fractions import Fraction

assert Fraction("3/4") == Fraction(3, 4), "from fraction string"
assert Fraction("0.5") == Fraction(1, 2), "from decimal string"
assert Fraction("-3/4") == Fraction(-3, 4), "negative fraction string"
assert Fraction("  3/4  ") == Fraction(3, 4), "surrounding whitespace stripped"

print("construct_from_string OK")
"###);
    assert_output(&out, r###"construct_from_string OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/fractions/copy_deepcopy_roundtrip.py`.
#[test]
fn test_gen_behavior_std_libs_fractions_copy_deepcopy_roundtrip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fractions"
# dimension = "behavior"
# case = "copy_deepcopy_roundtrip"
# subject = "fractions.Fraction"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_fractions.py"
# status = "filled"
# ///
"""fractions.Fraction: copy.copy and copy.deepcopy return an equal Fraction (immutable value semantics)"""
import copy
from fractions import Fraction

f = Fraction(3, 4)
assert copy.copy(f) == f, "shallow copy equals original"
assert copy.deepcopy(f) == f, "deep copy equals original"

print("copy_deepcopy_roundtrip OK")
"###);
    assert_output(&out, r###"copy_deepcopy_roundtrip OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/fractions/denominator_always_positive.py`.
#[test]
fn test_gen_behavior_std_libs_fractions_denominator_always_positive() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fractions"
# dimension = "behavior"
# case = "denominator_always_positive"
# subject = "fractions.Fraction"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_fractions.py"
# status = "filled"
# ///
"""fractions.Fraction: the denominator is always positive: 1/-2 normalizes the sign onto the numerator (-1/2)"""
from fractions import Fraction

assert Fraction(1, -2).numerator == -1, "sign moves to numerator"
assert Fraction(1, -2).denominator == 2, "denominator is positive"
assert Fraction(-3, -4).numerator == 3, "double negative is positive"
assert Fraction(-3, -4).denominator == 4, "double negative denom positive"

print("denominator_always_positive OK")
"###);
    assert_output(&out, r###"denominator_always_positive OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/fractions/divmod_floordiv_mod.py`.
#[test]
fn test_gen_behavior_std_libs_fractions_divmod_floordiv_mod() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fractions"
# dimension = "behavior"
# case = "divmod_floordiv_mod"
# subject = "fractions.Fraction"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_fractions.py"
# status = "filled"
# ///
"""fractions.Fraction: floor division, modulo and divmod stay exact: 7/2 // 2 == 1, 7/2 % 2 == 3/2, divmod(7/2, 2) == (1, 3/2)"""
from fractions import Fraction

assert Fraction(7, 2) // 2 == 1, "7/2 // 2 == 1"
assert Fraction(7, 2) % 2 == Fraction(3, 2), "7/2 % 2 == 3/2"
assert divmod(Fraction(7, 2), 2) == (1, Fraction(3, 2)), "divmod(7/2, 2)"

print("divmod_floordiv_mod OK")
"###);
    assert_output(&out, r###"divmod_floordiv_mod OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/fractions/equality_with_int_and_float.py`.
#[test]
fn test_gen_behavior_std_libs_fractions_equality_with_int_and_float() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fractions"
# dimension = "behavior"
# case = "equality_with_int_and_float"
# subject = "fractions.Fraction"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_fractions.py"
# status = "filled"
# ///
"""fractions.Fraction: a Fraction compares equal across numeric types: Fraction(6,3) == 2 and == 2.0, and Fraction(1,2) == 0.5"""
from fractions import Fraction

assert Fraction(6, 3) == 2, "Fraction(6, 3) == int 2"
assert Fraction(6, 3) == 2.0, "Fraction(6, 3) == float 2.0"
assert Fraction(1, 2) == 0.5, "Fraction(1, 2) == 0.5"
assert Fraction(1, 2) == Fraction(2, 4), "equality after reduction"

print("equality_with_int_and_float OK")
"###);
    assert_output(&out, r###"equality_with_int_and_float OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/fractions/exact_decimal_arithmetic.py`.
#[test]
fn test_gen_behavior_std_libs_fractions_exact_decimal_arithmetic() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fractions"
# dimension = "behavior"
# case = "exact_decimal_arithmetic"
# subject = "fractions.Fraction"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_fractions.py"
# status = "filled"
# ///
"""fractions.Fraction: exact arithmetic with no float error: 1/10 + 2/10 == 3/10 exactly, unlike float 0.1 + 0.2 != 0.3"""
from fractions import Fraction

assert Fraction(1, 10) + Fraction(2, 10) == Fraction(3, 10), "0.1 + 0.2 == 0.3 exact"
# Contrast: binary float arithmetic is not exact here.
assert 0.1 + 0.2 != 0.3, "float 0.1 + 0.2 != 0.3 (contrast)"
assert Fraction(1, 3) + Fraction(1, 6) == Fraction(1, 2), "1/3 + 1/6 == 1/2"

print("exact_decimal_arithmetic OK")
"###);
    assert_output(&out, r###"exact_decimal_arithmetic OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/fractions/floor_ceil_trunc_round.py`.
#[test]
fn test_gen_behavior_std_libs_fractions_floor_ceil_trunc_round() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fractions"
# dimension = "behavior"
# case = "floor_ceil_trunc_round"
# subject = "fractions.Fraction"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_fractions.py"
# status = "filled"
# ///
"""fractions.Fraction: math.floor/ceil/trunc and round operate on a Fraction: floor(7/3)=2, ceil(7/3)=3, trunc(-7/3)=-2, round(5/2)=2 (banker's rounding)"""
import math
from fractions import Fraction

assert math.floor(Fraction(7, 3)) == 2, f"floor(7/3) = {math.floor(Fraction(7, 3))!r}"
assert math.ceil(Fraction(7, 3)) == 3, f"ceil(7/3) = {math.ceil(Fraction(7, 3))!r}"
assert math.trunc(Fraction(-7, 3)) == -2, f"trunc(-7/3) = {math.trunc(Fraction(-7, 3))!r}"
# round uses banker's rounding (round-half-to-even).
assert round(Fraction(5, 2)) == 2, f"round(5/2) = {round(Fraction(5, 2))!r}"

print("floor_ceil_trunc_round OK")
"###);
    assert_output(&out, r###"floor_ceil_trunc_round OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/fractions/format_fixed_point.py`.
#[test]
fn test_gen_behavior_std_libs_fractions_format_fixed_point() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fractions"
# dimension = "behavior"
# case = "format_fixed_point"
# subject = "fractions.Fraction.__format__"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_fractions.py"
# status = "filled"
# ///
"""fractions.Fraction.__format__: Fraction supports presentation-type formatting: format(Fraction(1,3), '.4f') == '0.3333' and format(Fraction(1,2), '.2f') == '0.50'"""
from fractions import Fraction

assert format(Fraction(1, 3), ".4f") == "0.3333", f"1/3 .4f = {format(Fraction(1, 3), '.4f')!r}"
assert format(Fraction(1, 2), ".2f") == "0.50", f"1/2 .2f = {format(Fraction(1, 2), '.2f')!r}"

print("format_fixed_point OK")
"###);
    assert_output(&out, r###"format_fixed_point OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/fractions/hash_matches_equal_values.py`.
#[test]
fn test_gen_behavior_std_libs_fractions_hash_matches_equal_values() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fractions"
# dimension = "behavior"
# case = "hash_matches_equal_values"
# subject = "fractions.Fraction"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_fractions.py"
# status = "filled"
# ///
"""fractions.Fraction: hash agrees with == across numeric types so Fractions key dicts correctly: hash(Fraction(6,3)) == hash(2) == hash(2.0)"""
from fractions import Fraction

assert hash(Fraction(6, 3)) == hash(2), "hash matches equal int"
assert hash(Fraction(6, 3)) == hash(2.0), "hash matches equal float"
assert hash(Fraction(1, 2)) == hash(0.5), "hash matches equal 0.5"
# Equal values therefore collapse to one dict key.
assert len({Fraction(6, 3): "a", 2: "b"}) == 1, "equal Fraction/int share a key"

print("hash_matches_equal_values OK")
"###);
    assert_output(&out, r###"hash_matches_equal_values OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/fractions/int_float_bool_conversions.py`.
#[test]
fn test_gen_behavior_std_libs_fractions_int_float_bool_conversions() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fractions"
# dimension = "behavior"
# case = "int_float_bool_conversions"
# subject = "fractions.Fraction"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_fractions.py"
# status = "filled"
# ///
"""fractions.Fraction: numeric conversions truncate toward zero / convert exactly: int(7/3)==2, float(1/4)==0.25, bool(Fraction(0)) is False, bool(1/2) is True"""
from fractions import Fraction

assert int(Fraction(7, 3)) == 2, "int truncates toward zero"
assert int(Fraction(-7, 3)) == -2, "int truncates negative toward zero"
assert float(Fraction(1, 4)) == 0.25, "float conversion"
assert bool(Fraction(0)) is False, "zero is falsy"
assert bool(Fraction(1, 2)) is True, "non-zero is truthy"

print("int_float_bool_conversions OK")
"###);
    assert_output(&out, r###"int_float_bool_conversions OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/fractions/is_immutable_slots.py`.
#[test]
fn test_gen_behavior_std_libs_fractions_is_immutable_slots() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fractions"
# dimension = "behavior"
# case = "is_immutable_slots"
# subject = "fractions.Fraction"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_fractions.py"
# status = "filled"
# ///
"""fractions.Fraction: Fraction uses __slots__: instances reject arbitrary attribute assignment, numerator/denominator are read-only, and there is no instance __dict__"""
from fractions import Fraction

r = Fraction(13, 7)

# Arbitrary attribute assignment is rejected (no instance __dict__).
_raised = False
try:
    r.extra = 10
except AttributeError:
    _raised = True
assert _raised, "setting an unknown attribute raises AttributeError"

# numerator / denominator are read-only properties.
_ro = False
try:
    r.numerator = 1
except AttributeError:
    _ro = True
assert _ro, "numerator is read-only"

# Instances expose no __dict__ thanks to __slots__.
assert not hasattr(r, "__dict__"), "Fraction instance has no __dict__"

print("is_immutable_slots OK")
"###);
    assert_output(&out, r###"is_immutable_slots OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/fractions/mixed_arithmetic_with_int.py`.
#[test]
fn test_gen_behavior_std_libs_fractions_mixed_arithmetic_with_int() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fractions"
# dimension = "behavior"
# case = "mixed_arithmetic_with_int"
# subject = "fractions.Fraction"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_fractions.py"
# status = "filled"
# ///
"""fractions.Fraction: arithmetic mixes with int and reduces to a plain int when whole: Fraction(1,2) + 1 == 3/2 and Fraction(3,4) * 4 == 3"""
from fractions import Fraction

assert Fraction(1, 2) + 1 == Fraction(3, 2), "Fraction + int"
assert Fraction(3, 4) * 4 == 3, "Fraction * int reduces to whole int value"
assert 1 - Fraction(1, 4) == Fraction(3, 4), "int - Fraction (reflected)"

print("mixed_arithmetic_with_int OK")
"###);
    assert_output(&out, r###"mixed_arithmetic_with_int OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/fractions/numbers_abc_membership.py`.
#[test]
fn test_gen_behavior_std_libs_fractions_numbers_abc_membership() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fractions"
# dimension = "behavior"
# case = "numbers_abc_membership"
# subject = "fractions.Fraction"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_fractions.py"
# status = "filled"
# ///
"""fractions.Fraction: a Fraction registers as a numbers.Rational (and therefore Number) but not numbers.Integral"""
import numbers
from fractions import Fraction

assert isinstance(Fraction(1, 2), numbers.Rational), "Fraction is Rational"
assert isinstance(Fraction(1, 2), numbers.Number), "Fraction is Number"
assert not isinstance(Fraction(1, 2), numbers.Integral), "Fraction is not Integral"

print("numbers_abc_membership OK")
"###);
    assert_output(&out, r###"numbers_abc_membership OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/fractions/ordering_with_int_and_float.py`.
#[test]
fn test_gen_behavior_std_libs_fractions_ordering_with_int_and_float() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fractions"
# dimension = "behavior"
# case = "ordering_with_int_and_float"
# subject = "fractions.Fraction"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_fractions.py"
# status = "filled"
# ///
"""fractions.Fraction: Fraction ordering mixes with numeric types: 1/3 < 0.5, 2/3 > 0.5, and equality after reduction (1/2 == 2/4) with 1/3 < 1/2 < 3/4"""
from fractions import Fraction

assert Fraction(1, 3) < 0.5, "1/3 < 0.5"
assert Fraction(2, 3) > 0.5, "2/3 > 0.5"
assert Fraction(1, 3) < Fraction(1, 2) < Fraction(3, 4), "1/3 < 1/2 < 3/4"
assert Fraction(1, 2) == Fraction(2, 4), "1/2 == 2/4 after reduction"

print("ordering_with_int_and_float OK")
"###);
    assert_output(&out, r###"ordering_with_int_and_float OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/fractions/power_positive_and_negative.py`.
#[test]
fn test_gen_behavior_std_libs_fractions_power_positive_and_negative() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fractions"
# dimension = "behavior"
# case = "power_positive_and_negative"
# subject = "fractions.Fraction"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_fractions.py"
# status = "filled"
# ///
"""fractions.Fraction: integer powers stay exact: (2/3)**2 == 4/9 and (2/3)**-1 == 3/2"""
from fractions import Fraction

assert Fraction(2, 3) ** 2 == Fraction(4, 9), f"(2/3)^2 = {Fraction(2, 3) ** 2!r}"
assert Fraction(2, 3) ** -1 == Fraction(3, 2), f"(2/3)^-1 = {Fraction(2, 3) ** -1!r}"

print("power_positive_and_negative OK")
"###);
    assert_output(&out, r###"power_positive_and_negative OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/fractions/preserves_rational_component_types.py`.
#[test]
fn test_gen_behavior_std_libs_fractions_preserves_rational_component_types() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fractions"
# dimension = "behavior"
# case = "preserves_rational_component_types"
# subject = "fractions.Fraction"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_fractions.py"
# status = "filled"
# ///
"""fractions.Fraction: built from objects exposing .numerator/.denominator, Fraction divides through by the gcd but keeps the original component type rather than coercing to plain int (int subclass MyInt)"""
from fractions import Fraction


class MyInt(int):
    """An int that returns its own type from arithmetic and components."""

    def __mul__(self, other):
        return type(self)(int(self) * int(other))

    def __floordiv__(self, other):
        return type(self)(int(self) // int(other))

    def __mod__(self, other):
        return type(self)(int(self) % int(other))

    @property
    def numerator(self):
        return type(self)(int(self))

    @property
    def denominator(self):
        return type(self)(1)


# 3/6 reduces to 1/2 while retaining MyInt components.
f = Fraction(MyInt(1 * 3), MyInt(2 * 3))
assert f.numerator == 1, f"numerator value = {f.numerator!r}"
assert f.denominator == 2, f"denominator value = {f.denominator!r}"
assert type(f.numerator) is MyInt, f"numerator type = {type(f.numerator)!r}"
assert type(f.denominator) is MyInt, f"denominator type = {type(f.denominator)!r}"

print("preserves_rational_component_types OK")
"###);
    assert_output(&out, r###"preserves_rational_component_types OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/fractions/reduces_to_lowest_terms.py`.
#[test]
fn test_gen_behavior_std_libs_fractions_reduces_to_lowest_terms() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fractions"
# dimension = "behavior"
# case = "reduces_to_lowest_terms"
# subject = "fractions.Fraction"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_fractions.py"
# status = "filled"
# ///
"""fractions.Fraction: Fraction always reduces to lowest terms: 12/8 -> 3/2 and -4/6 -> -2/3"""
from fractions import Fraction

assert Fraction(12, 8).numerator == 3, f"12/8 num = {Fraction(12, 8).numerator!r}"
assert Fraction(12, 8).denominator == 2, f"12/8 den = {Fraction(12, 8).denominator!r}"
assert Fraction(-4, 6).numerator == -2, f"-4/6 num = {Fraction(-4, 6).numerator!r}"
assert Fraction(-4, 6).denominator == 3, f"-4/6 den = {Fraction(-4, 6).denominator!r}"

print("reduces_to_lowest_terms OK")
"###);
    assert_output(&out, r###"reduces_to_lowest_terms OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/fractions/str_and_repr.py`.
#[test]
fn test_gen_behavior_std_libs_fractions_str_and_repr() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fractions"
# dimension = "behavior"
# case = "str_and_repr"
# subject = "fractions.Fraction"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_fractions.py"
# status = "filled"
# ///
"""fractions.Fraction: str shows num/den (or the bare integer when denom is 1) and repr shows Fraction(n, d): str(3/4)=='3/4', str(Fraction(5))=='5', repr(3/4)=='Fraction(3, 4)'"""
from fractions import Fraction

assert str(Fraction(3, 4)) == "3/4", f"str = {str(Fraction(3, 4))!r}"
assert str(Fraction(5)) == "5", f"whole str = {str(Fraction(5))!r}"
assert repr(Fraction(3, 4)) == "Fraction(3, 4)", f"repr = {repr(Fraction(3, 4))!r}"

print("str_and_repr OK")
"###);
    assert_output(&out, r###"str_and_repr OK
"###);
}
