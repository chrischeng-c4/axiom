use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/pep/fstrings/ascii_conversion_escapes_non_ascii.py`.
#[test]
fn test_gen_behavior_pep_fstrings_ascii_conversion_escapes_non_ascii() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "fstrings"
# dimension = "behavior"
# case = "ascii_conversion_escapes_non_ascii"
# subject = "fstring.conversion"
# kind = "semantic"
# xfail = "mamba diverges on the !a ASCII conversion of a non-ASCII string (retired surface.py head comment: AssertionError !a = \"'cafe'\"; project_mamba_pep_silent_divergences_2026_05_27)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.conversion: !a escapes non-ASCII characters via ascii(): f'{"caf\\u00e9"!a}' contains the backslash-escaped code point '\\\\u00e9'"""
# !a converts via ascii(), backslash-escaping non-ASCII code points

unicode_str = "caf\u00e9"
ascii_result = f"{unicode_str!a}"
assert "\\u00e9" in ascii_result or "\\xe9" in ascii_result, f"!a = {ascii_result!r}"

print("ascii_conversion_escapes_non_ascii OK")
"###);
    assert_output(&out, r###"ascii_conversion_escapes_non_ascii OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/fstrings/builtin_call_in_field.py`.
#[test]
fn test_gen_behavior_pep_fstrings_builtin_call_in_field() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "fstrings"
# dimension = "behavior"
# case = "builtin_call_in_field"
# subject = "fstring.expression"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.expression: a builtin call result interpolates: with data=[10,20,30], f'sum={sum(data)}' is 'sum=60'"""
# a builtin call is a valid field expression

data = [10, 20, 30]
total_str = f"sum={sum(data)}"
assert total_str == "sum=60", f"sum = {total_str!r}"

print("builtin_call_in_field OK")
"###);
    assert_output(&out, r###"builtin_call_in_field OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/fstrings/conditional_expression_in_field.py`.
#[test]
fn test_gen_behavior_pep_fstrings_conditional_expression_in_field() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "fstrings"
# dimension = "behavior"
# case = "conditional_expression_in_field"
# subject = "fstring.expression"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.expression: a conditional expression evaluates inside a field: with n=5, f"{'odd' if n % 2 else 'even'}" is 'odd'"""
# a ternary conditional is a valid field expression

n = 5
assert f"{'odd' if n % 2 else 'even'}" == "odd", "conditional"

print("conditional_expression_in_field OK")
"###);
    assert_output(&out, r###"conditional_expression_in_field OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/fstrings/conversion_flags_custom_dunders.py`.
#[test]
fn test_gen_behavior_pep_fstrings_conversion_flags_custom_dunders() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "fstrings"
# dimension = "behavior"
# case = "conversion_flags_custom_dunders"
# subject = "fstring.conversion"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.conversion: !r calls __repr__ and !s calls __str__ on a custom class: a class with __repr__ 'Custom(repr)' and __str__ 'Custom(str)' gives f'{c!r}' 'Custom(repr)' and f'{c!s}' 'Custom(str)'"""
# conversion flags dispatch the matching dunder on a custom type

class Custom:
    def __repr__(self) -> str:
        return "Custom(repr)"
    def __str__(self) -> str:
        return "Custom(str)"

c = Custom()
assert f"{c!r}" == "Custom(repr)", f"!r = {f'{c!r}'!r}"
assert f"{c!s}" == "Custom(str)", f"!s = {f'{c!s}'!r}"

print("conversion_flags_custom_dunders OK")
"###);
    assert_output(&out, r###"conversion_flags_custom_dunders OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/fstrings/conversion_flags_str_and_repr.py`.
#[test]
fn test_gen_behavior_pep_fstrings_conversion_flags_str_and_repr() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "fstrings"
# dimension = "behavior"
# case = "conversion_flags_str_and_repr"
# subject = "fstring.conversion"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.conversion: !s uses str and !r uses repr: with v="it's alive", f'{v!s}' is v and f'{v!r}' is repr(v)"""
# !s / !r conversion flags select str / repr before formatting

_val = "it's alive"
assert f"{_val!s}" == _val, "!s = str(val)"
assert f"{_val!r}" == repr(_val), "!r = repr(val)"

print("conversion_flags_str_and_repr OK")
"###);
    assert_output(&out, r###"conversion_flags_str_and_repr OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/fstrings/dict_subscript_in_field.py`.
#[test]
fn test_gen_behavior_pep_fstrings_dict_subscript_in_field() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "fstrings"
# dimension = "behavior"
# case = "dict_subscript_in_field"
# subject = "fstring.expression"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.expression: a dict subscript evaluates inside a field: with d={'key':'value'}, f'{d["key"]}' is 'value'"""
# an f-string index key is an evaluated subscript expression

d = {"key": "value"}
assert f"{d['key']}" == "value", "dict value in f-string"

print("dict_subscript_in_field OK")
"###);
    assert_output(&out, r###"dict_subscript_in_field OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/fstrings/doubled_braces_are_literal.py`.
#[test]
fn test_gen_behavior_pep_fstrings_doubled_braces_are_literal() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "fstrings"
# dimension = "behavior"
# case = "doubled_braces_are_literal"
# subject = "fstring.literal"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.literal: doubled braces produce single literal braces and never start a field: f'{{not interpolated}}' is '{not interpolated}'"""
# {{ and }} escape to literal braces

assert f"{{not interpolated}}" == "{not interpolated}", "escaped braces"

print("doubled_braces_are_literal OK")
"###);
    assert_output(&out, r###"doubled_braces_are_literal OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/fstrings/dynamic_format_spec_from_variable.py`.
#[test]
fn test_gen_behavior_pep_fstrings_dynamic_format_spec_from_variable() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "fstrings"
# dimension = "behavior"
# case = "dynamic_format_spec_from_variable"
# subject = "fstring.format_spec"
# kind = "semantic"
# xfail = "mamba diverges on a fully dynamic format spec built from a variable (retired behavior.py head comment: AssertionError dynamic fmt = '3.14159265'; project_mamba_pep_silent_divergences_2026_05_27)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.format_spec: the whole format spec is itself an expression: with fmt='.3f', pi=3.14159265, f'{pi:{fmt}}' is '3.142'"""
# the format spec after ':' is evaluated as an expression

fmt = ".3f"
pi = 3.14159265
result = f"{pi:{fmt}}"
assert result == "3.142", f"dynamic fmt = {result!r}"

print("dynamic_format_spec_from_variable OK")
"###);
    assert_output(&out, r###"dynamic_format_spec_from_variable OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/fstrings/dynamic_width_from_nested_field.py`.
#[test]
fn test_gen_behavior_pep_fstrings_dynamic_width_from_nested_field() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "fstrings"
# dimension = "behavior"
# case = "dynamic_width_from_nested_field"
# subject = "fstring.format_spec"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.format_spec: a nested {} inside the spec supplies the width dynamically: with width=10, f"{'hi':^{width}}" centres to '    hi    '"""
# a replacement field may appear inside the format spec

width = 10
centered = f"{'hi':^{width}}"
assert centered == "    hi    ", f"dynamic width = {centered!r}"

print("dynamic_width_from_nested_field OK")
"###);
    assert_output(&out, r###"dynamic_width_from_nested_field OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/fstrings/empty_fstring_is_empty.py`.
#[test]
fn test_gen_behavior_pep_fstrings_empty_fstring_is_empty() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "fstrings"
# dimension = "behavior"
# case = "empty_fstring_is_empty"
# subject = "fstring.literal"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.literal: an f-string with no fields and no text is the empty string: f'' is ''"""
# an f-string with no content is an ordinary empty literal

assert f"" == "", "empty f-string"

print("empty_fstring_is_empty OK")
"###);
    assert_output(&out, r###"empty_fstring_is_empty OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/fstrings/field_evaluated_in_enclosing_scope.py`.
#[test]
fn test_gen_behavior_pep_fstrings_field_evaluated_in_enclosing_scope() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "fstrings"
# dimension = "behavior"
# case = "field_evaluated_in_enclosing_scope"
# subject = "fstring.evaluation"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.evaluation: a field expression is evaluated in the enclosing scope with its side effects: a counter-incrementing call inside f'{next_val()}' yields '1' and bumps the module counter to 1"""
# replacement fields evaluate in the enclosing scope, side effects included

counter = 0


def next_val() -> int:
    global counter
    counter += 1
    return counter


s = f"{next_val()}"
assert s == "1", f"expr eval = {s!r}"
assert counter == 1, f"counter = {counter!r}"

print("field_evaluated_in_enclosing_scope OK")
"###);
    assert_output(&out, r###"field_evaluated_in_enclosing_scope OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/fstrings/float_precision_spec.py`.
#[test]
fn test_gen_behavior_pep_fstrings_float_precision_spec() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "fstrings"
# dimension = "behavior"
# case = "float_precision_spec"
# subject = "fstring.format_spec"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.format_spec: a float precision spec rounds: f'{3.14159:.2f}' is '3.14'"""
# a presentation-type format spec controls float rendering

assert f"{3.14159:.2f}" == "3.14", f"float fmt = {f'{3.14159:.2f}'!r}"

print("float_precision_spec OK")
"###);
    assert_output(&out, r###"float_precision_spec OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/fstrings/generator_join_workaround_for_backslash.py`.
#[test]
fn test_gen_behavior_pep_fstrings_generator_join_workaround_for_backslash() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "fstrings"
# dimension = "behavior"
# case = "generator_join_workaround_for_backslash"
# subject = "fstring.expression"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.expression: binding a join result before the field avoids a literal backslash in the field: with items=[1,2,3], joined=', '.join(str(i) for i in items), f'[{joined}]' is '[1, 2, 3]'"""
# bind a computed value first, then interpolate it

items = [1, 2, 3]
joined = ", ".join(str(i) for i in items)
assert f"[{joined}]" == "[1, 2, 3]", f"join = {f'[{joined}]'!r}"

print("generator_join_workaround_for_backslash OK")
"###);
    assert_output(&out, r###"generator_join_workaround_for_backslash OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/fstrings/hex_fill_and_width_spec.py`.
#[test]
fn test_gen_behavior_pep_fstrings_hex_fill_and_width_spec() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "fstrings"
# dimension = "behavior"
# case = "hex_fill_and_width_spec"
# subject = "fstring.format_spec"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.format_spec: an alternate-form zero-padded hex spec applies fill and width: f'{255:#010x}' is '0x000000ff'"""
# '#' alt-form, zero fill, and width compose in the format spec

assert f"{255:#010x}" == "0x000000ff", f"hex fmt = {f'{255:#010x}'!r}"

print("hex_fill_and_width_spec OK")
"###);
    assert_output(&out, r###"hex_fill_and_width_spec OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/fstrings/implicit_concatenation_of_fstrings.py`.
#[test]
fn test_gen_behavior_pep_fstrings_implicit_concatenation_of_fstrings() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "fstrings"
# dimension = "behavior"
# case = "implicit_concatenation_of_fstrings"
# subject = "fstring.literal"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.literal: adjacent f-string literals concatenate at compile time: f'part1' f'-part2' f'-part3' is 'part1-part2-part3'"""
# implicit string-literal concatenation works across f-string parts

long = (
    f"part1"
    f"-part2"
    f"-part3"
)
assert long == "part1-part2-part3", f"long = {long!r}"

print("implicit_concatenation_of_fstrings OK")
"###);
    assert_output(&out, r###"implicit_concatenation_of_fstrings OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/fstrings/inline_arithmetic_expression.py`.
#[test]
fn test_gen_behavior_pep_fstrings_inline_arithmetic_expression() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "fstrings"
# dimension = "behavior"
# case = "inline_arithmetic_expression"
# subject = "fstring.expression"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.expression: an arithmetic expression evaluates inline: with x=10, f'{x + 5}' is '15'"""
# replacement fields hold arbitrary expressions

x = 10
assert f"{x + 5}" == "15", f"arith = {f'{x + 5}'!r}"

print("inline_arithmetic_expression OK")
"###);
    assert_output(&out, r###"inline_arithmetic_expression OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/fstrings/integer_base_format_codes.py`.
#[test]
fn test_gen_behavior_pep_fstrings_integer_base_format_codes() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "fstrings"
# dimension = "behavior"
# case = "integer_base_format_codes"
# subject = "fstring.format_spec"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.format_spec: integer presentation codes render the base: f'{255:b}' is '11111111', f'{255:o}' is '377', f'{255:x}' is 'ff', f'{255:X}' is 'FF'"""
# b/o/x/X presentation types render an int in binary/octal/hex

assert f"{255:b}" == "11111111", "binary"
assert f"{255:o}" == "377", "octal"
assert f"{255:x}" == "ff", "hex lower"
assert f"{255:X}" == "FF", "hex upper"

print("integer_base_format_codes OK")
"###);
    assert_output(&out, r###"integer_base_format_codes OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/fstrings/interpolates_name_value.py`.
#[test]
fn test_gen_behavior_pep_fstrings_interpolates_name_value() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "fstrings"
# dimension = "behavior"
# case = "interpolates_name_value"
# subject = "fstring.interpolation"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.interpolation: a bare name field interpolates its value: with x=10, f'{x}' is '10'"""
# f-string interpolation is syntax; no import needed

x = 10
assert f"{x}" == "10", f"basic = {f'{x}'!r}"

print("interpolates_name_value OK")
"###);
    assert_output(&out, r###"interpolates_name_value OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/fstrings/method_call_in_field.py`.
#[test]
fn test_gen_behavior_pep_fstrings_method_call_in_field() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "fstrings"
# dimension = "behavior"
# case = "method_call_in_field"
# subject = "fstring.expression"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.expression: a method call inside a field is evaluated and formatted: f"{'hello'.upper()}" is 'HELLO'"""
# a call expression in a field is evaluated and formatted

assert f"{'hello'.upper()}" == "HELLO", "method call in f-string"

print("method_call_in_field OK")
"###);
    assert_output(&out, r###"method_call_in_field OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/fstrings/multiple_fields_with_arithmetic.py`.
#[test]
fn test_gen_behavior_pep_fstrings_multiple_fields_with_arithmetic() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "fstrings"
# dimension = "behavior"
# case = "multiple_fields_with_arithmetic"
# subject = "fstring.interpolation"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.interpolation: multiple fields interleave with literal text and a derived field: with a=3, b=4, f'{a} + {b} = {a + b}' is '3 + 4 = 7'"""
# fields and literal runs concatenate left to right

a, b = 3, 4
assert f"{a} + {b} = {a + b}" == "3 + 4 = 7", "multiple"

print("multiple_fields_with_arithmetic OK")
"###);
    assert_output(&out, r###"multiple_fields_with_arithmetic OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/fstrings/nested_field_supplies_sign_option.py`.
#[test]
fn test_gen_behavior_pep_fstrings_nested_field_supplies_sign_option() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "fstrings"
# dimension = "behavior"
# case = "nested_field_supplies_sign_option"
# subject = "fstring.format_spec"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.format_spec: a nested {} inside the spec supplies the sign option: with sign='+', f'{42:{sign}d}' is '+42'"""
# a nested field can supply any part of the format spec

sign = "+"
assert f"{42:{sign}d}" == "+42", f"sign format = {f'{42:{sign}d}'!r}"

print("nested_field_supplies_sign_option OK")
"###);
    assert_output(&out, r###"nested_field_supplies_sign_option OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/fstrings/nested_fstring_is_just_an_expression.py`.
#[test]
fn test_gen_behavior_pep_fstrings_nested_fstring_is_just_an_expression() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "fstrings"
# dimension = "behavior"
# case = "nested_fstring_is_just_an_expression"
# subject = "fstring.nesting"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.nesting: an inner f-string is just another expression in a field: inner = f"{'world'}" then f'hello {inner}' is 'hello world'"""
# f-strings nest because a field holds any expression

inner = f"{'world'}"
assert f"hello {inner}" == "hello world", f"nested f = {f'hello {inner}'!r}"

print("nested_fstring_is_just_an_expression OK")
"###);
    assert_output(&out, r###"nested_fstring_is_just_an_expression OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/fstrings/underscore_grouping_spec.py`.
#[test]
fn test_gen_behavior_pep_fstrings_underscore_grouping_spec() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "fstrings"
# dimension = "behavior"
# case = "underscore_grouping_spec"
# subject = "fstring.format_spec"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.format_spec: the '_' grouping option inserts underscores every three digits: f'{1000000:_}' is '1_000_000'"""
# '_' is a thousands-grouping option in the format spec

assert f"{1000000:_}" == "1_000_000", "underscore sep"

print("underscore_grouping_spec OK")
"###);
    assert_output(&out, r###"underscore_grouping_spec OK
"###);
}
