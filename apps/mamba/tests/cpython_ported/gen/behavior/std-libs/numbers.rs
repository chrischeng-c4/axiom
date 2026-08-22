use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/numbers/concrete_builtin_registrations.py`.
#[test]
fn test_gen_behavior_std_libs_numbers_concrete_builtin_registrations() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "numbers"
# dimension = "behavior"
# case = "concrete_builtin_registrations"
# subject = "numbers.Integral"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_abc.py"
# status = "filled"
# ///
"""numbers.Integral: int registers as Integral, float as Real (not Integral), complex as Complex (not Real), bool as Integral, and str is not a Number"""
import numbers

# The built-in numeric types are registered at exactly their tower rung.
assert isinstance(1, numbers.Integral), "int is Integral"
assert isinstance(1.0, numbers.Real), "float is Real"
assert not isinstance(1.0, numbers.Integral), "float is NOT Integral"
assert isinstance(1j, numbers.Complex), "complex is Complex"
assert not isinstance(1j, numbers.Real), "complex is NOT Real"
assert isinstance(True, numbers.Integral), "bool is Integral (bool subclasses int)"

# A non-numeric type is not anywhere in the tower.
assert not isinstance("x", numbers.Number), "str is not a Number"

print("concrete_builtin_registrations OK")
"###);
    assert_output(&out, r###"concrete_builtin_registrations OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/numbers/fraction_is_rational_not_integral.py`.
#[test]
fn test_gen_behavior_std_libs_numbers_fraction_is_rational_not_integral() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "numbers"
# dimension = "behavior"
# case = "fraction_is_rational_not_integral"
# subject = "numbers.Rational"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_fractions.py"
# status = "filled"
# ///
"""numbers.Rational: fractions.Fraction is a Rational and a Real but not an Integral"""
import numbers
from fractions import Fraction

half = Fraction(1, 2)

# Fraction is registered exactly at the Rational rung.
assert isinstance(half, numbers.Rational), "Fraction is Rational"
assert isinstance(half, numbers.Real), "Rational is also Real"
assert isinstance(half, numbers.Complex), "Rational is also Complex"
assert isinstance(half, numbers.Number), "Rational is also Number"

# But a fraction is not an integer, so it stops above the Integral rung.
assert not isinstance(half, numbers.Integral), "Fraction is NOT Integral"

print("fraction_is_rational_not_integral OK")
"###);
    assert_output(&out, r###"fraction_is_rational_not_integral OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/numbers/number_has_no_abstracts_instantiable.py`.
#[test]
fn test_gen_behavior_std_libs_numbers_number_has_no_abstracts_instantiable() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "numbers"
# dimension = "behavior"
# case = "number_has_no_abstracts_instantiable"
# subject = "numbers.Number"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_abc.py"
# status = "filled"
# ///
"""numbers.Number: Number declares no abstract methods, so it is concrete and Number() instantiates without raising"""
import numbers

# Number is the tower root and declares zero abstract methods, so unlike the
# four lower ABCs it is concrete and instantiates with no TypeError.
assert numbers.Number.__abstractmethods__ == frozenset(), numbers.Number.__abstractmethods__

instance = numbers.Number()
assert isinstance(instance, numbers.Number), type(instance)

print("number_has_no_abstracts_instantiable OK")
"###);
    assert_output(&out, r###"number_has_no_abstracts_instantiable OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/numbers/tower_abcs_are_callable.py`.
#[test]
fn test_gen_behavior_std_libs_numbers_tower_abcs_are_callable() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "numbers"
# dimension = "behavior"
# case = "tower_abcs_are_callable"
# subject = "numbers.Number"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_abc.py"
# status = "filled"
# ///
"""numbers.Number: all five tower ABCs (Number/Complex/Real/Rational/Integral) are callable class objects, not function-name stubs (#1261)"""
import numbers

# Regression guard for #1261: the five ABC entries must be real callable class
# objects, not function-name string stubs that AttributeError on reference.
for abc in (numbers.Number, numbers.Complex, numbers.Real,
            numbers.Rational, numbers.Integral):
    assert callable(abc), abc

# They are distinct names along the tower, not aliases of one stub.
names = {numbers.Number.__name__, numbers.Complex.__name__, numbers.Real.__name__,
         numbers.Rational.__name__, numbers.Integral.__name__}
assert names == {"Number", "Complex", "Real", "Rational", "Integral"}, names

print("tower_abcs_are_callable OK")
"###);
    assert_output(&out, r###"tower_abcs_are_callable OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/numbers/tower_inheritance_chain.py`.
#[test]
fn test_gen_behavior_std_libs_numbers_tower_inheritance_chain() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "numbers"
# dimension = "behavior"
# case = "tower_inheritance_chain"
# subject = "numbers.Integral"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_abc.py"
# status = "filled"
# ///
"""numbers.Integral: the numeric tower nests Integral < Rational < Real < Complex < Number via __bases__, issubclass, and the Integral MRO"""
import numbers

# Each rung's direct base is the next one up the tower (recovered from the
# errors.py integral_bases probe, generalized to the whole chain).
assert [b.__name__ for b in numbers.Integral.__bases__] == ["Rational"]
assert [b.__name__ for b in numbers.Rational.__bases__] == ["Real"]
assert [b.__name__ for b in numbers.Real.__bases__] == ["Complex"]
assert [b.__name__ for b in numbers.Complex.__bases__] == ["Number"]

# issubclass reflects the full nesting.
assert issubclass(numbers.Integral, numbers.Rational)
assert issubclass(numbers.Rational, numbers.Real)
assert issubclass(numbers.Real, numbers.Complex)
assert issubclass(numbers.Complex, numbers.Number)
assert issubclass(int, numbers.Integral)

# The Integral MRO walks the whole tower down to object.
assert [c.__name__ for c in numbers.Integral.__mro__] == [
    "Integral", "Rational", "Real", "Complex", "Number", "object",
], [c.__name__ for c in numbers.Integral.__mro__]

print("tower_inheritance_chain OK")
"###);
    assert_output(&out, r###"tower_inheritance_chain OK
"###);
}
