use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/decimal/compare_with_int.py`.
#[test]
fn test_gen_behavior_std_libs_decimal_compare_with_int() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "behavior"
# case = "compare_with_int"
# subject = "decimal.Decimal"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_decimal.py"
# status = "filled"
# ///
"""decimal.Decimal: Decimal compares against plain int by mathematical value: Decimal('10') == 10, > 9, < 11"""
from decimal import Decimal

# Decimal compares against a plain int by mathematical value.
assert Decimal("10") == 10, "Decimal == int"
assert Decimal("10") > 9, "Decimal > int"
assert Decimal("10") < 11, "Decimal < int"

print("compare_with_int OK")
"###);
    assert_output(&out, r###"compare_with_int OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/decimal/construct_from_bool.py`.
#[test]
fn test_gen_behavior_std_libs_decimal_construct_from_bool() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "behavior"
# case = "construct_from_bool"
# subject = "decimal.Decimal"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_decimal.py"
# status = "filled"
# ///
"""decimal.Decimal: Decimal(True) == Decimal(1), Decimal(False) == Decimal(0), and bool(Decimal) reflects nonzero-ness"""
from decimal import Decimal

# From bool: True == 1, False == 0; truthiness of Decimal.
assert Decimal(True) == Decimal(1), "Decimal(True)"
assert Decimal(False) == Decimal(0), "Decimal(False)"
assert bool(Decimal(0)) is False, "bool(Decimal(0))"
assert bool(Decimal("0.372")) is True, "bool(nonzero)"

print("construct_from_bool OK")
"###);
    assert_output(&out, r###"construct_from_bool OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/decimal/construct_from_int.py`.
#[test]
fn test_gen_behavior_std_libs_decimal_construct_from_int() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "behavior"
# case = "construct_from_int"
# subject = "decimal.Decimal"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_decimal.py"
# status = "filled"
# ///
"""decimal.Decimal: Decimal from int is exact and sign-preserving across 45, -45, large, and 0"""
from decimal import Decimal

# From int: exact, sign preserved, arbitrary precision.
assert str(Decimal(45)) == "45", "int 45"
assert str(Decimal(-45)) == "-45", "int -45"
assert str(Decimal(500000123)) == "500000123", "int large"
assert str(Decimal(0)) == "0", "int 0"

print("construct_from_int OK")
"###);
    assert_output(&out, r###"construct_from_int OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/decimal/construct_from_tuple.py`.
#[test]
fn test_gen_behavior_std_libs_decimal_construct_from_tuple() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "behavior"
# case = "construct_from_tuple"
# subject = "decimal.Decimal"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_decimal.py"
# status = "filled"
# ///
"""decimal.Decimal: Decimal((sign, digits, exponent)) builds the value, and a 'F' exponent yields Infinity"""
from decimal import Decimal

# From tuple (sign, digits, exponent); 'F' exponent = Infinity.
assert str(Decimal((0, (0,), 0))) == "0", "tuple 0"
assert str(Decimal((1, (4, 5), 0))) == "-45", "tuple -45"
assert str(Decimal((0, (4, 5, 3, 4), -2))) == "45.34", "tuple 45.34"
assert str(Decimal((0, (), "F"))) == "Infinity", "tuple Infinity"

print("construct_from_tuple OK")
"###);
    assert_output(&out, r###"construct_from_tuple OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/decimal/context_keyword_scopes_flags.py`.
#[test]
fn test_gen_behavior_std_libs_decimal_context_keyword_scopes_flags() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "behavior"
# case = "context_keyword_scopes_flags"
# subject = "decimal.Context"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_decimal.py"
# status = "filled"
# ///
"""decimal.Context: the context= keyword scopes precision/flags to a private Context: an overflow inside it sets that context's flag, leaving the active context untouched"""
from decimal import Decimal, Context, localcontext, Overflow

D = Decimal
# Decimal methods accept a context= keyword that scopes precision/flags
# without touching the active context.
xc = Context(prec=1, Emax=1, Emin=-1)
with localcontext() as active:
    active.clear_flags()
    assert D(9, context=xc) == 9, "constructor context= keyword"
    assert D("9.73").normalize(context=xc) == D("1E+1"), "normalize context= keyword"
    assert D("0.0625").sqrt(context=xc) == D("0.2"), "sqrt context= keyword"
    # An error inside xc sets xc's flag, not the active context's.
    xc.clear_flags()
    try:
        D(8).exp(context=xc)
        raise AssertionError("exp overflow should raise")
    except Overflow:
        pass
    assert xc.flags[Overflow], "overflow flag set on xc"
    assert not active.flags[Overflow], "active context flag untouched"

print("context_keyword_scopes_flags OK")
"###);
    assert_output(&out, r###"context_keyword_scopes_flags OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/decimal/context_method_int_coercion.py`.
#[test]
fn test_gen_behavior_std_libs_decimal_context_method_int_coercion() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "behavior"
# case = "context_method_int_coercion"
# subject = "decimal.Context"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_decimal.py"
# status = "filled"
# ///
"""decimal.Context: Context arithmetic/utility methods coerce int operands to Decimal: to_eng_string/normalize/to_integral_value/copy_decimal/number_class agree on int vs Decimal input"""
from decimal import Decimal, Context

D = Decimal
c = Context()
# Context arithmetic/utility methods coerce int operands to Decimal.
assert c.to_eng_string(10) == c.to_eng_string(D(10)), "to_eng_string int"
assert c.normalize(10) == c.normalize(D(10)), "normalize int"
assert c.to_integral_value(10) == c.to_integral_value(D(10)), "to_integral_value int"
assert c.copy_decimal(-1) == c.copy_decimal(D(-1)), "copy_decimal int"
assert c.number_class(123) == c.number_class(D(123)), "number_class int"

print("context_method_int_coercion OK")
"###);
    assert_output(&out, r###"context_method_int_coercion OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/decimal/cross_type_vs_float.py`.
#[test]
fn test_gen_behavior_std_libs_decimal_cross_type_vs_float() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "behavior"
# case = "cross_type_vs_float"
# subject = "decimal.Decimal"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_decimal.py"
# status = "filled"
# ///
"""decimal.Decimal: Decimal vs float compares by value: Decimal('0.25') == 0.25 and < 3.0, but Decimal('0.1') != float 0.1 (binary inexactness)"""
from decimal import Decimal

da = Decimal("0.25")
db = Decimal("3.0")
# Decimal vs float: ordering and equality compare by mathematical value.
assert da < 3.0 and db > 0.25, "ordering vs float"
assert da == 0.25, "Decimal('0.25') == 0.25"
# 0.1 is not exactly representable as binary float, so it differs from the
# exact Decimal('0.1').
assert Decimal("0.1") != 0.1, "Decimal('0.1') != float 0.1"

print("cross_type_vs_float OK")
"###);
    assert_output(&out, r###"cross_type_vs_float OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/decimal/exact_arithmetic_no_float_drift.py`.
#[test]
fn test_gen_behavior_std_libs_decimal_exact_arithmetic_no_float_drift() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "behavior"
# case = "exact_arithmetic_no_float_drift"
# subject = "decimal.Decimal"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_decimal.py"
# status = "filled"
# ///
"""decimal.Decimal: Decimal('0.1') + Decimal('0.2') == Decimal('0.3') exactly (str is '0.3'), unlike binary float where 0.1+0.2 != 0.3"""
from decimal import Decimal

# Decimal arithmetic is exact — it avoids the binary-float representation error.
assert Decimal("0.1") + Decimal("0.2") == Decimal("0.3"), "0.1+0.2 == 0.3"
assert str(Decimal("0.1") + Decimal("0.2")) == "0.3", f"str(0.1+0.2) = {str(Decimal('0.1') + Decimal('0.2'))!r}"
# Contrast: the same sum in binary float drifts off 0.3.
assert 0.1 + 0.2 != 0.3, "float 0.1+0.2 != 0.3 (contrast)"
print("exact_arithmetic_no_float_drift OK")
"###);
    assert_output(&out, r###"exact_arithmetic_no_float_drift OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/decimal/exact_large_int_multiply.py`.
#[test]
fn test_gen_behavior_std_libs_decimal_exact_large_int_multiply() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "behavior"
# case = "exact_large_int_multiply"
# subject = "decimal.Decimal"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_decimal.py"
# status = "filled"
# ///
"""decimal.Decimal: Decimal multiplication is exact for large integers: 123456789 * 987654321 == 121932631112635269"""
from decimal import Decimal

_m = Decimal("123456789") * Decimal("987654321")
assert str(_m) == "121932631112635269", f"large int multiply = {str(_m)!r}"

print("exact_large_int_multiply OK")
"###);
    assert_output(&out, r###"exact_large_int_multiply OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/decimal/format_align_fill.py`.
#[test]
fn test_gen_behavior_std_libs_decimal_format_align_fill() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "behavior"
# case = "format_align_fill"
# subject = "decimal.Decimal"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_decimal.py"
# status = "filled"
# ///
"""decimal.Decimal: format alignment and fill: default right align, <, >, ^, custom fill char, and fill-after-sign with '='"""
from decimal import Decimal


def f(value, spec):
    return format(Decimal(value), spec)


# Alignment and fill: <, >, ^, and a custom fill character.
assert f("123", "6") == "   123", "default right align"
assert f("123", "<6") == "123   ", "left align"
assert f("123", "^6") == " 123  ", "center align"
assert f("123", "?^5") == "?123?", "custom fill center"
assert f("-45.6", "/=10") == "-/////45.6", "fill after sign"

print("format_align_fill OK")
"###);
    assert_output(&out, r###"format_align_fill OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/decimal/format_fixed.py`.
#[test]
fn test_gen_behavior_std_libs_decimal_format_fixed() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "behavior"
# case = "format_fixed"
# subject = "decimal.Decimal"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_decimal.py"
# status = "filled"
# ///
"""decimal.Decimal: the 'f' format code uses fixed notation, keeps trailing zeros from scale, and rounds/pads at explicit precision"""
from decimal import Decimal


def f(value, spec):
    return format(Decimal(value), spec)


# Fixed notation: 'f' with explicit precision rounds/pads.
assert f("3.2E2", "f") == "320", "f no fraction"
assert f("3.200E2", "f") == "320.0", "f keeps trailing"
assert f("3.14159265", ".4f") == "3.1416", "f precision"

print("format_fixed OK")
"###);
    assert_output(&out, r###"format_fixed OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/decimal/format_general.py`.
#[test]
fn test_gen_behavior_std_libs_decimal_format_general() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "behavior"
# case = "format_general"
# subject = "decimal.Decimal"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_decimal.py"
# status = "filled"
# ///
"""decimal.Decimal: the 'g' format code switches between fixed and scientific and honors precision"""
from decimal import Decimal


def f(value, spec):
    return format(Decimal(value), spec)


# General: 'g' switches between fixed and scientific.
assert f("0E1", "g") == "0e+1", "g sci zero"
assert f("3.14159265", ".5g") == "3.1416", "g precision"

print("format_general OK")
"###);
    assert_output(&out, r###"format_general OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/decimal/format_percent.py`.
#[test]
fn test_gen_behavior_std_libs_decimal_format_percent() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "behavior"
# case = "format_percent"
# subject = "decimal.Decimal"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_decimal.py"
# status = "filled"
# ///
"""decimal.Decimal: the '%' format code scales by 100 and appends '%'"""
from decimal import Decimal


def f(value, spec):
    return format(Decimal(value), spec)


# Percent: '%' scales by 100 and appends '%'.
assert f("2.34", ".3%") == "234.000%", "percent"
assert f("1.23", ".0%") == "123%", "percent no fraction"

print("format_percent OK")
"###);
    assert_output(&out, r###"format_percent OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/decimal/format_scientific.py`.
#[test]
fn test_gen_behavior_std_libs_decimal_format_scientific() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "behavior"
# case = "format_scientific"
# subject = "decimal.Decimal"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_decimal.py"
# status = "filled"
# ///
"""decimal.Decimal: the 'e' format code normalizes to scientific notation with a signed exponent and rounds at the requested precision"""
from decimal import Decimal


def f(value, spec):
    return format(Decimal(value), spec)


# Scientific notation: 'e' normalizes the exponent with a sign.
assert f("1.5", "e") == "1.5e+0", "e basic"
assert f("0.015", "e") == "1.5e-2", "e small"
assert f("9.9999999", ".6e") == "1.000000e+1", "e precision rounds"

print("format_scientific OK")
"###);
    assert_output(&out, r###"format_scientific OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/decimal/format_sign_control.py`.
#[test]
fn test_gen_behavior_std_libs_decimal_format_sign_control() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "behavior"
# case = "format_sign_control"
# subject = "decimal.Decimal"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_decimal.py"
# status = "filled"
# ///
"""decimal.Decimal: the '+' sign flag forces a leading sign with padding under '='"""
from decimal import Decimal


def f(value, spec):
    return format(Decimal(value), spec)


# Sign control: '+' forces a sign, padded under '='.
assert f("123", "=+6") == "+  123", "plus sign with pad"

print("format_sign_control OK")
"###);
    assert_output(&out, r###"format_sign_control OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/decimal/format_special_values.py`.
#[test]
fn test_gen_behavior_std_libs_decimal_format_special_values() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "behavior"
# case = "format_special_values"
# subject = "decimal.Decimal"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_decimal.py"
# status = "filled"
# ///
"""decimal.Decimal: special values format as words (NaN/Infinity) and honor sign and alignment"""
from decimal import Decimal


def f(value, spec):
    return format(Decimal(value), spec)


# Special values format as words and honor sign/fill.
assert f("NaN", "e") == "NaN", "NaN passthrough"
assert f("Inf", ".3e") == "Infinity", "Infinity word"
assert f("-Inf", ">10") == " -Infinity", "Infinity aligned"

print("format_special_values OK")
"###);
    assert_output(&out, r###"format_special_values OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/decimal/format_thousands_separator.py`.
#[test]
fn test_gen_behavior_std_libs_decimal_format_thousands_separator() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "behavior"
# case = "format_thousands_separator"
# subject = "decimal.Decimal"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_decimal.py"
# status = "filled"
# ///
"""decimal.Decimal: the ',' format code groups thousands, works on negatives, and combines with zero-padding"""
from decimal import Decimal


def f(value, spec):
    return format(Decimal(value), spec)


# Thousands separator: ','.
assert f("1234567", ",") == "1,234,567", "comma grouping"
assert f("-123456", ",") == "-123,456", "comma negative"
assert f("1234.56", "09,") == "01,234.56", "zero-pad with comma"

print("format_thousands_separator OK")
"###);
    assert_output(&out, r###"format_thousands_separator OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/decimal/format_z_negative_zero.py`.
#[test]
fn test_gen_behavior_std_libs_decimal_format_z_negative_zero() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "behavior"
# case = "format_z_negative_zero"
# subject = "decimal.Decimal"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_decimal.py"
# status = "filled"
# ///
"""decimal.Decimal: the 'z' format code coerces a negative-zero result to positive zero but keeps a real negative"""
from decimal import Decimal


def f(value, spec):
    return format(Decimal(value), spec)


# 'z' coerces a negative-zero result to positive zero but keeps a real negative.
assert f("-0.", "z.1f") == "0.0", "z negative-zero coercion"
assert f("-.09", "z.1f") == "-0.1", "z keeps real negative"

print("format_z_negative_zero OK")
"###);
    assert_output(&out, r###"format_z_negative_zero OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/decimal/immutable_copy_returns_same.py`.
#[test]
fn test_gen_behavior_std_libs_decimal_immutable_copy_returns_same() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "behavior"
# case = "immutable_copy_returns_same"
# subject = "decimal.Decimal"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_decimal.py"
# status = "filled"
# ///
"""decimal.Decimal: Decimal is immutable: copy.copy and copy.deepcopy return the same object"""
import copy
from decimal import Decimal

# Decimal is immutable: copy and deepcopy return the same object.
d = Decimal("43.24")
assert copy.copy(d) is d, "copy.copy returns same object"
assert copy.deepcopy(d) is d, "copy.deepcopy returns same object"

print("immutable_copy_returns_same OK")
"###);
    assert_output(&out, r###"immutable_copy_returns_same OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/decimal/infinity_arithmetic.py`.
#[test]
fn test_gen_behavior_std_libs_decimal_infinity_arithmetic() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "behavior"
# case = "infinity_arithmetic"
# subject = "decimal.Decimal"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_decimal.py"
# status = "filled"
# ///
"""decimal.Decimal: Infinity arithmetic: inf + 1000 == Infinity, -inf == -Infinity, and inf > any finite value"""
from decimal import Decimal

_inf = Decimal("Infinity")
assert _inf + Decimal("1000") == Decimal("Infinity"), "inf + 1000 = inf"
assert -_inf == Decimal("-Infinity"), "-inf"
assert _inf > Decimal("999999"), "inf > large number"

print("infinity_arithmetic OK")
"###);
    assert_output(&out, r###"infinity_arithmetic OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/decimal/int_and_trunc_toward_zero.py`.
#[test]
fn test_gen_behavior_std_libs_decimal_int_and_trunc_toward_zero() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "behavior"
# case = "int_and_trunc_toward_zero"
# subject = "decimal.Decimal"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_decimal.py"
# status = "filled"
# ///
"""decimal.Decimal: int() and math.trunc() truncate toward zero and agree with float's int(): int(Decimal('3.99'))==3, int(Decimal('-3.99'))==-3"""
import math
from decimal import Decimal

D = Decimal
# int() and math.trunc() truncate toward zero and agree with float's int().
for x in (-250, -1, 0, 1, 137, 249):
    s = "%0.2f" % (x / 100.0)
    assert int(D(s)) == int(float(s)), f"int(Decimal({s!r}))"
    assert math.trunc(D(s)) == int(D(s)), f"trunc(Decimal({s!r}))"
assert int(D("3.99")) == 3, "int truncates fraction"
assert int(D("-3.99")) == -3, "int truncates toward zero"

print("int_and_trunc_toward_zero OK")
"###);
    assert_output(&out, r###"int_and_trunc_toward_zero OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/decimal/localcontext_keyword_overrides.py`.
#[test]
fn test_gen_behavior_std_libs_decimal_localcontext_keyword_overrides() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "behavior"
# case = "localcontext_keyword_overrides"
# subject = "decimal.localcontext"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_decimal.py"
# status = "filled"
# ///
"""decimal.localcontext: localcontext accepts keyword overrides (prec/rounding/Emin/Emax) and applies them to the yielded context"""
from decimal import localcontext, ROUND_HALF_DOWN

# localcontext accepts keyword overrides and applies them to the yielded ctx.
with localcontext(prec=10, rounding=ROUND_HALF_DOWN, Emin=-20, Emax=20) as ctx:
    assert ctx.prec == 10 and ctx.rounding == ROUND_HALF_DOWN, "kw overrides applied"

print("localcontext_keyword_overrides OK")
"###);
    assert_output(&out, r###"localcontext_keyword_overrides OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/decimal/localcontext_restores_and_copies.py`.
#[test]
fn test_gen_behavior_std_libs_decimal_localcontext_restores_and_copies() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "behavior"
# case = "localcontext_restores_and_copies"
# subject = "decimal.localcontext"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_decimal.py"
# status = "filled"
# ///
"""decimal.localcontext: localcontext yields a fresh copy that is the active context inside the block, then restores the previous context exactly on exit"""
from decimal import getcontext, localcontext

# localcontext yields a fresh copy and restores the previous context on exit.
orig = getcontext()
with localcontext() as entered:
    inside = getcontext()
    assert inside is entered, "getcontext() inside is the entered context"
    assert orig is not entered, "entered context is a copy"
assert getcontext() is orig, "context restored on exit"

print("localcontext_restores_and_copies OK")
"###);
    assert_output(&out, r###"localcontext_restores_and_copies OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/decimal/number_class_subnormal.py`.
#[test]
fn test_gen_behavior_std_libs_decimal_number_class_subnormal() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "behavior"
# case = "number_class_subnormal"
# subject = "decimal.Context.number_class"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_decimal.py"
# status = "filled"
# ///
"""decimal.Context.number_class: number_class names the IEEE category: under a tight context Decimal('0.01').number_class() == '+Subnormal'"""
from decimal import Decimal, Context

D = Decimal
# number_class names the IEEE category of a value under a tight context.
xc = Context(prec=1, Emax=1, Emin=-1)
assert D("0.01").number_class(context=xc) == "+Subnormal", "number_class subnormal"

print("number_class_subnormal OK")
"###);
    assert_output(&out, r###"number_class_subnormal OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/decimal/prec_controls_division.py`.
#[test]
fn test_gen_behavior_std_libs_decimal_prec_controls_division() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "behavior"
# case = "prec_controls_division"
# subject = "decimal.localcontext"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_decimal.py"
# status = "filled"
# ///
"""decimal.localcontext: a localcontext with prec=4 rounds 1/3 to '0.3333'"""
from decimal import Decimal, localcontext

# A context's prec controls default precision of inexact results.
with localcontext() as _ctx:
    _ctx.prec = 4
    _result = Decimal("1") / Decimal("3")
    assert str(_result) == "0.3333", f"prec=4 division = {str(_result)!r}"

print("prec_controls_division OK")
"###);
    assert_output(&out, r###"prec_controls_division OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/decimal/quantize_half_even_bankers.py`.
#[test]
fn test_gen_behavior_std_libs_decimal_quantize_half_even_bankers() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "behavior"
# case = "quantize_half_even_bankers"
# subject = "decimal.Decimal.quantize"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_decimal.py"
# status = "filled"
# ///
"""decimal.Decimal.quantize: ROUND_HALF_EVEN (banker's rounding) rounds 2.5 -> '2' and 3.5 -> '4' to the unit place"""
from decimal import Decimal, ROUND_HALF_EVEN

_r2 = Decimal("2.5").quantize(Decimal("1"), rounding=ROUND_HALF_EVEN)
assert str(_r2) == "2", f"2.5 HALF_EVEN = {str(_r2)!r}"
_r3 = Decimal("3.5").quantize(Decimal("1"), rounding=ROUND_HALF_EVEN)
assert str(_r3) == "4", f"3.5 HALF_EVEN = {str(_r3)!r}"

print("quantize_half_even_bankers OK")
"###);
    assert_output(&out, r###"quantize_half_even_bankers OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/decimal/quantize_half_up.py`.
#[test]
fn test_gen_behavior_std_libs_decimal_quantize_half_up() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "behavior"
# case = "quantize_half_up"
# subject = "decimal.Decimal.quantize"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_decimal.py"
# status = "filled"
# ///
"""decimal.Decimal.quantize: quantize(Decimal('0.01'), rounding=ROUND_HALF_UP) rounds 3.14159 to '3.14'"""
from decimal import Decimal, ROUND_HALF_UP

_q = Decimal("3.14159").quantize(Decimal("0.01"), rounding=ROUND_HALF_UP)
assert str(_q) == "3.14", f"quantize ROUND_HALF_UP = {str(_q)!r}"

print("quantize_half_up OK")
"###);
    assert_output(&out, r###"quantize_half_up OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/decimal/quantize_round_down_truncates.py`.
#[test]
fn test_gen_behavior_std_libs_decimal_quantize_round_down_truncates() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "behavior"
# case = "quantize_round_down_truncates"
# subject = "decimal.Decimal.quantize"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_decimal.py"
# status = "filled"
# ///
"""decimal.Decimal.quantize: ROUND_DOWN truncates toward zero: 3.9 -> '3' and -3.9 -> '-3' to the unit place"""
from decimal import Decimal, ROUND_DOWN

_rd = Decimal("3.9").quantize(Decimal("1"), rounding=ROUND_DOWN)
assert str(_rd) == "3", f"3.9 ROUND_DOWN = {str(_rd)!r}"
_rdn = Decimal("-3.9").quantize(Decimal("1"), rounding=ROUND_DOWN)
assert str(_rdn) == "-3", f"-3.9 ROUND_DOWN = {str(_rdn)!r}"

print("quantize_round_down_truncates OK")
"###);
    assert_output(&out, r###"quantize_round_down_truncates OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/decimal/repr_eval_round_trip.py`.
#[test]
fn test_gen_behavior_std_libs_decimal_repr_eval_round_trip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "behavior"
# case = "repr_eval_round_trip"
# subject = "decimal.Decimal"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_decimal.py"
# status = "filled"
# ///
"""decimal.Decimal: repr(Decimal) round-trips through eval back to an equal Decimal for several tuple-built values"""
from decimal import Decimal

# repr round-trips through eval back to an equal Decimal.
for _src in [(0, (0,), 0), (1, (4, 5), 0), (0, (4, 5, 3, 4), -2)]:
    _d = Decimal(_src)
    assert _d == eval(repr(_d)), f"eval(repr) round trip {_src}"

print("repr_eval_round_trip OK")
"###);
    assert_output(&out, r###"repr_eval_round_trip OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/decimal/unary_operators.py`.
#[test]
fn test_gen_behavior_std_libs_decimal_unary_operators() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "behavior"
# case = "unary_operators"
# subject = "decimal.Decimal"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_decimal.py"
# status = "filled"
# ///
"""decimal.Decimal: unary +, -, and abs() behave as expected on Decimal(45)/Decimal(-45)"""
from decimal import Decimal

D = Decimal
assert +D(45) == D(45), "unary plus"
assert -D(45) == D(-45), "unary minus"
assert abs(D(-45)) == D(45), "abs"

print("unary_operators OK")
"###);
    assert_output(&out, r###"unary_operators OK
"###);
}
