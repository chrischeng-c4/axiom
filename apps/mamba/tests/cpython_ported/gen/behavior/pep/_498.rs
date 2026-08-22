use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/pep/498/conversion_flags_repr_and_str.py`.
#[test]
fn test_gen_behavior_pep_498_conversion_flags_repr_and_str() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "498"
# dimension = "behavior"
# case = "conversion_flags_repr_and_str"
# subject = "fstring.conversion"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.conversion: !r uses repr and !s uses str: f"{'x'!r}" is "'x'" and f'{3 != 4!s}' is 'True'"""
# !r / !s conversion flags select repr / str before formatting

assert f"{'x'!r}" == "'x'"
assert f"{3 != 4!s}" == "True"

print("conversion_flags_repr_and_str OK")
"###);
    assert_output(&out, r###"conversion_flags_repr_and_str OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/498/conversion_then_format_spec.py`.
#[test]
fn test_gen_behavior_pep_498_conversion_then_format_spec() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "498"
# dimension = "behavior"
# case = "conversion_then_format_spec"
# subject = "fstring.conversion"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.conversion: a conversion then a format spec apply in order: f'{3 != 4!s:.3}' converts True via str then truncates to 'Tru'"""
# conversion runs before the format spec

assert f"{3 != 4!s:.3}" == "Tru"

print("conversion_then_format_spec OK")
"###);
    assert_output(&out, r###"conversion_then_format_spec OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/498/custom_format_dispatch_on_spec.py`.
#[test]
fn test_gen_behavior_pep_498_custom_format_dispatch_on_spec() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "498"
# dimension = "behavior"
# case = "custom_format_dispatch_on_spec"
# subject = "fstring.format_spec"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.format_spec: __format__ receives the raw spec: a Marker whose __format__ returns '*' when spec is empty else the spec gives f'{m}' '*', f'{m:}' '*', f'{m:x}' 'x'"""
# the format spec string is passed verbatim to __format__

class Marker:
    def __format__(self, spec):
        return "*" if not spec else spec

m = Marker()
assert f"{m}" == "*"
assert f"{m:}" == "*"
assert f"{m:x}" == "x"

print("custom_format_dispatch_on_spec OK")
"###);
    assert_output(&out, r###"custom_format_dispatch_on_spec OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/498/debug_eq_emits_expr_and_value.py`.
#[test]
fn test_gen_behavior_pep_498_debug_eq_emits_expr_and_value() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "498"
# dimension = "behavior"
# case = "debug_eq_emits_expr_and_value"
# subject = "fstring.debug"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.debug: {expr=} emits the source text, '=', then the value (repr by default): with x=10, f'{x=}' is 'x=10' and f'{name=}' is "name='world'" for name='world'"""
# the =-debug form echoes the expression source plus its value

x = 10
assert f"{x=}" == "x=10"
name = "world"
assert f"{name=}" == "name='world'"

print("debug_eq_emits_expr_and_value OK")
"###);
    assert_output(&out, r###"debug_eq_emits_expr_and_value OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/498/debug_eq_preserves_internal_spacing.py`.
#[test]
fn test_gen_behavior_pep_498_debug_eq_preserves_internal_spacing() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "498"
# dimension = "behavior"
# case = "debug_eq_preserves_internal_spacing"
# subject = "fstring.debug"
# kind = "semantic"
# xfail = "mamba strips the expression name/spacing from the {expr = } debug form (project_mamba_pep_silent_divergences_2026_05_27)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.debug: the =-debug form echoes verbatim spacing/operators: f'{x = }' is 'x = 10', f'1 == 2={1 == 2!r}' is '1 == 2=False', f'{1 + 2 = }' is '1 + 2 = 3', f'{total*2=}' is 'total*2=6'"""
# whitespace and operators inside the debug field are echoed verbatim

x = 10
# Surrounding spaces around `=` are preserved verbatim in the text.
assert f"{x = }" == "x = 10"
# The echoed expression text is the verbatim source between the braces,
# including any internal whitespace and operators (gh-129093).
assert f"1==2={1 == 2!r}" == "1==2=False"
assert f"1 == 2={1 == 2!r}" == "1 == 2=False"
assert f"1!=2={1 != 2!r}" == "1!=2=True"
assert f"1 != 2={1 != 2!r}" == "1 != 2=True"
assert f"(1*2) != (3)={1 * 2 != 3!r}" == "(1*2) != (3)=True"
total = 1 + 2
assert f"{1 + 2 = }" == "1 + 2 = 3"
assert f"{total*2=}" == "total*2=6"

print("debug_eq_preserves_internal_spacing OK")
"###);
    assert_output(&out, r###"debug_eq_preserves_internal_spacing OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/498/debug_eq_with_conversion_and_spec.py`.
#[test]
fn test_gen_behavior_pep_498_debug_eq_with_conversion_and_spec() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "498"
# dimension = "behavior"
# case = "debug_eq_with_conversion_and_spec"
# subject = "fstring.debug"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.debug: an explicit conversion or spec overrides the =-debug default repr: with val=3.14159, f'{val=:.2f}' is 'val=3.14' and f'{name=!s}' is 'name=world' for name='world'"""
# !s or a format spec override the default repr in a debug field

val = 3.14159
name = "world"
assert f"{val=:.2f}" == "val=3.14"
assert f"{name=!s}" == "name=world"

print("debug_eq_with_conversion_and_spec OK")
"###);
    assert_output(&out, r###"debug_eq_with_conversion_and_spec OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/498/doubled_braces_are_literal.py`.
#[test]
fn test_gen_behavior_pep_498_doubled_braces_are_literal() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "498"
# dimension = "behavior"
# case = "doubled_braces_are_literal"
# subject = "fstring.literal"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.literal: doubled braces produce a single literal brace and never start a field: f'{{1+1}}' is '{1+1}' and f'{{1+1' is '{1+1'"""
# {{ and }} escape to literal braces

assert f"{{1+1}}" == "{1+1}"
assert f"{{1+1" == "{1+1"

print("doubled_braces_are_literal OK")
"###);
    assert_output(&out, r###"doubled_braces_are_literal OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/498/empty_and_whitespace_literals.py`.
#[test]
fn test_gen_behavior_pep_498_empty_and_whitespace_literals() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "498"
# dimension = "behavior"
# case = "empty_and_whitespace_literals"
# subject = "fstring.literal"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.literal: an f-string with no fields is an ordinary literal: f'' is '', f' ' is ' ', f'a' is 'a'"""
# f-string literal portions behave like plain string literals

assert f"" == ""
assert f" " == " "
assert f"a" == "a"

print("empty_and_whitespace_literals OK")
"###);
    assert_output(&out, r###"empty_and_whitespace_literals OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/498/empty_spec_equals_no_spec.py`.
#[test]
fn test_gen_behavior_pep_498_empty_spec_equals_no_spec() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "498"
# dimension = "behavior"
# case = "empty_spec_equals_no_spec"
# subject = "fstring.format_spec"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.format_spec: an empty format spec equals no spec for str: f'{x}', f'{x:}', f'{x!s:}' all give 'test' and f'{x!r:}' gives "'test'" for x='test'"""
# {x:} dispatches __format__('') just like {x}

x = "test"
assert f"{x}" == "test"
assert f"{x:}" == "test"          # empty spec is the same as no spec
assert f"{x!s:}" == "test"
assert f"{x!r:}" == "'test'"
# Built-in numbers ignore an empty spec and stringify normally.
assert f"{3:}" == "3"
assert f"{3!s:}" == "3"

print("empty_spec_equals_no_spec OK")
"###);
    assert_output(&out, r###"empty_spec_equals_no_spec OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/498/field_captures_closure_variable.py`.
#[test]
fn test_gen_behavior_pep_498_field_captures_closure_variable() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "498"
# dimension = "behavior"
# case = "field_captures_closure_variable"
# subject = "fstring.scope"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.scope: a nested function's f-string captures the enclosing variable: closure('987')() is 'x:987' and closure(7)() is 'x:7'"""
# an f-string field closes over enclosing-function variables

def closure(x):
    # A nested function's f-string captures the enclosing variable.
    def inner():
        return f"x:{x}"
    return inner

assert closure("987")() == "x:987"
assert closure(7)() == "x:7"

print("field_captures_closure_variable OK")
"###);
    assert_output(&out, r###"field_captures_closure_variable OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/498/field_sees_locals_and_shadowing.py`.
#[test]
fn test_gen_behavior_pep_498_field_sees_locals_and_shadowing() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "498"
# dimension = "behavior"
# case = "field_sees_locals_and_shadowing"
# subject = "fstring.scope"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.scope: a field reads locals and a local shadows the same-named global within the function"""
# a local binding shadows a global of the same name in a field

a_global = "global variable"

def uses_local():
    a_local = "local variable"
    return f"g:{a_global} l:{a_local}"

def shadows_global():
    # A local of the same name shadows the global within the function.
    a_global = "really a local"
    return f"g:{a_global!r}"

assert uses_local() == "g:global variable l:local variable"
assert shadows_global() == "g:'really a local'"

print("field_sees_locals_and_shadowing OK")
"###);
    assert_output(&out, r###"field_sees_locals_and_shadowing OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/498/field_sees_module_globals.py`.
#[test]
fn test_gen_behavior_pep_498_field_sees_module_globals() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "498"
# dimension = "behavior"
# case = "field_sees_module_globals"
# subject = "fstring.scope"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.scope: an f-string field reads module globals: a function returning f'g:{a_global}' sees the module-level a_global"""
# replacement fields resolve names in the enclosing scope chain

a_global = "global variable"

def uses_global():
    # An f-string sees module globals.
    return f"g:{a_global}"

assert uses_global() == "g:global variable"

print("field_sees_module_globals OK")
"###);
    assert_output(&out, r###"field_sees_module_globals OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/498/fields_evaluated_left_to_right.py`.
#[test]
fn test_gen_behavior_pep_498_fields_evaluated_left_to_right() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "498"
# dimension = "behavior"
# case = "fields_evaluated_left_to_right"
# subject = "fstring.evaluation"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.evaluation: replacement fields evaluate left to right: a Counter whose __format__ returns an incrementing count gives f'{c} {c}' == '1 2'"""
# field evaluation order is left to right, observable via side effects

class Counter:
    def __init__(self):
        self.i = 0
    def __format__(self, spec):
        self.i += 1
        return str(self.i)

c = Counter()
assert f"{c} {c}" == "1 2"

print("fields_evaluated_left_to_right OK")
"###);
    assert_output(&out, r###"fields_evaluated_left_to_right OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/498/function_call_result_interpolates.py`.
#[test]
fn test_gen_behavior_pep_498_function_call_result_interpolates() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "498"
# dimension = "behavior"
# case = "function_call_result_interpolates"
# subject = "fstring.expression"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.expression: a function-call result interpolates: with label(v) returning 'x='+str(v), f'{label(10)}' is 'x=10'"""
# a call expression in a field is evaluated and formatted

def label(v):
    return "x=" + str(v)

assert f"{label(10)}" == "x=10"

print("function_call_result_interpolates OK")
"###);
    assert_output(&out, r###"function_call_result_interpolates OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/498/inline_arithmetic_and_comparison_expressions.py`.
#[test]
fn test_gen_behavior_pep_498_inline_arithmetic_and_comparison_expressions() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "498"
# dimension = "behavior"
# case = "inline_arithmetic_and_comparison_expressions"
# subject = "fstring.expression"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.expression: arbitrary expressions evaluate inline: f'{x*2}' is '20' (x=10), f'{3 + 4}' is '7', f'{0 == 1}' is 'False', f'{3 != 4}' is 'True'"""
# replacement fields hold arbitrary expressions

x = 10
assert f"{x*2}" == "20"
assert f"{3 + 4}" == "7"
assert f"{0 == 1}" == "False"
assert f"{3 != 4}" == "True"

print("inline_arithmetic_and_comparison_expressions OK")
"###);
    assert_output(&out, r###"inline_arithmetic_and_comparison_expressions OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/498/inner_quotes_differ_from_delimiter.py`.
#[test]
fn test_gen_behavior_pep_498_inner_quotes_differ_from_delimiter() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "498"
# dimension = "behavior"
# case = "inner_quotes_differ_from_delimiter"
# subject = "fstring.quoting"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.quoting: a field's string literal may use a different quote style, including inside a triple-quoted f-string: f'''{d["'"]}''' is 'squote' and f'''{d['"']}''' is 'dquote'"""
# an f-string field may contain differently-quoted string literals

d = {'"': "dquote", "'": "squote", "foo": "bar"}
assert f"""{d["'"]}""" == "squote"
assert f"""{d['"']}""" == "dquote"
assert f"{d['foo']}" == "bar"

print("inner_quotes_differ_from_delimiter OK")
"###);
    assert_output(&out, r###"inner_quotes_differ_from_delimiter OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/498/interpolates_name_and_format_spec.py`.
#[test]
fn test_gen_behavior_pep_498_interpolates_name_and_format_spec() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "498"
# dimension = "behavior"
# case = "interpolates_name_and_format_spec"
# subject = "fstring.interpolation"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.interpolation: a bare name field and a name with a format spec interpolate: f'{x}' is '42' and f'{x:04d}' is '0042' for x = 42"""
# f-string interpolation is syntax; no import needed

x = 42
assert f"{x}" == "42"
assert f"{x:04d}" == "0042"

print("interpolates_name_and_format_spec OK")
"###);
    assert_output(&out, r###"interpolates_name_and_format_spec OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/498/leading_fstring_is_not_a_docstring.py`.
#[test]
fn test_gen_behavior_pep_498_leading_fstring_is_not_a_docstring() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "498"
# dimension = "behavior"
# case = "leading_fstring_is_not_a_docstring"
# subject = "fstring.statement"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.statement: an f-string as a function's first statement is not a docstring: fn.__doc__ is None"""
# only a plain str literal as the first statement is a docstring

def fn():
    f"not a docstring"

assert fn.__doc__ is None

print("leading_fstring_is_not_a_docstring OK")
"###);
    assert_output(&out, r###"leading_fstring_is_not_a_docstring OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/498/line_continuation_collapses.py`.
#[test]
fn test_gen_behavior_pep_498_line_continuation_collapses() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "498"
# dimension = "behavior"
# case = "line_continuation_collapses"
# subject = "fstring.literal"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.literal: a line-continuation backslash collapses to nothing even inside an f-string: eval('f"\\\\\\n"') is ''"""
# a trailing backslash-newline is a line continuation

assert eval('f"\\\n"') == ""

print("line_continuation_collapses OK")
"###);
    assert_output(&out, r###"line_continuation_collapses OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/498/literal_backslashes_behave_normally.py`.
#[test]
fn test_gen_behavior_pep_498_literal_backslashes_behave_normally() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "498"
# dimension = "behavior"
# case = "literal_backslashes_behave_normally"
# subject = "fstring.literal"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.literal: backslashes in the literal portion behave like a normal string: f'\\\\' is '\\\\' and f'\\\\\\\\' is '\\\\\\\\'"""
# backslash escapes in literal runs match plain string literals

assert f"\\" == "\\"
assert f"\\\\" == "\\\\"

print("literal_backslashes_behave_normally OK")
"###);
    assert_output(&out, r###"literal_backslashes_behave_normally OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/498/loop_variable_resolves_freshly.py`.
#[test]
fn test_gen_behavior_pep_498_loop_variable_resolves_freshly() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "498"
# dimension = "behavior"
# case = "loop_variable_resolves_freshly"
# subject = "fstring.scope"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.scope: a loop variable resolves freshly each iteration: f'i:{i}' over range(5) yields ['i:0','i:1','i:2','i:3','i:4']"""
# a field re-reads the loop variable on each pass

seen = []
for i in range(5):
    seen.append(f"i:{i}")
assert seen == ["i:0", "i:1", "i:2", "i:3", "i:4"]

print("loop_variable_resolves_freshly OK")
"###);
    assert_output(&out, r###"loop_variable_resolves_freshly OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/498/multiple_fields_mix_with_literal_text.py`.
#[test]
fn test_gen_behavior_pep_498_multiple_fields_mix_with_literal_text() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "498"
# dimension = "behavior"
# case = "multiple_fields_mix_with_literal_text"
# subject = "fstring.interpolation"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.interpolation: multiple fields interleave with literal text: with a=98, b='abc', f'X{a}Y{b}Z' is 'X98YabcZ'"""
# fields and literal runs concatenate left to right

a, b = 98, "abc"
assert f"X{a}Y{b}Z" == "X98YabcZ"

print("multiple_fields_mix_with_literal_text OK")
"###);
    assert_output(&out, r###"multiple_fields_mix_with_literal_text OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/498/nested_field_supplies_dynamic_width.py`.
#[test]
fn test_gen_behavior_pep_498_nested_field_supplies_dynamic_width() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "498"
# dimension = "behavior"
# case = "nested_field_supplies_dynamic_width"
# subject = "fstring.format_spec"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.format_spec: a nested {} inside the spec supplies the width dynamically: with width=10, f"x={'foo'*2:{width}}" is 'x=foofoo    ' and f'{10*2:{width}}' is '        20'"""
# a replacement field may appear inside the format spec

y = 2
width = 10
assert f"x={'foo' * y:{width}}" == "x=foofoo    "
assert f"{10 * y:{width}}" == "        20"

print("nested_field_supplies_dynamic_width OK")
"###);
    assert_output(&out, r###"nested_field_supplies_dynamic_width OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/498/nested_fstring_is_just_an_expression.py`.
#[test]
fn test_gen_behavior_pep_498_nested_fstring_is_just_an_expression() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "498"
# dimension = "behavior"
# case = "nested_fstring_is_just_an_expression"
# subject = "fstring.nesting"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.nesting: an inner f-string is just another expression: with y=5, f"{f'{y}' * 3}" is '555'"""
# f-strings nest because a field holds any expression

y = 5
assert f"{f'{y}' * 3}" == "555"

print("nested_fstring_is_just_an_expression OK")
"###);
    assert_output(&out, r###"nested_fstring_is_just_an_expression OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/498/raw_fstring_keeps_backslashes.py`.
#[test]
fn test_gen_behavior_pep_498_raw_fstring_keeps_backslashes() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "498"
# dimension = "behavior"
# case = "raw_fstring_keeps_backslashes"
# subject = "fstring.prefix"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.prefix: raw f-strings keep backslashes verbatim: rf'\\n' is '\\\\n' and rf'{1}\\t' is '1\\\\t'"""
# the r prefix disables backslash interpretation in the literal run

assert rf"\n" == "\\n"
assert rf"{1}\t" == "1\\t"

print("raw_fstring_keeps_backslashes OK")
"###);
    assert_output(&out, r###"raw_fstring_keeps_backslashes OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/498/raw_fstring_prefix_orders.py`.
#[test]
fn test_gen_behavior_pep_498_raw_fstring_prefix_orders() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "498"
# dimension = "behavior"
# case = "raw_fstring_prefix_orders"
# subject = "fstring.prefix"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.prefix: both prefix orders combine raw + f: rf'\\n{1}' is '\\\\n1' and fr'{1}\\t' is '1\\\\t'"""
# rf and fr prefixes both yield a raw f-string

assert f"{1}" == "1"
assert rf"\n{1}" == "\\n1"
assert fr"{1}\t" == "1\\t"

print("raw_fstring_prefix_orders OK")
"###);
    assert_output(&out, r###"raw_fstring_prefix_orders OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/498/trailing_double_brace_is_literal_after_format.py`.
#[test]
fn test_gen_behavior_pep_498_trailing_double_brace_is_literal_after_format() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "498"
# dimension = "behavior"
# case = "trailing_double_brace_is_literal_after_format"
# subject = "fstring.format_spec"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.format_spec: the spec is read greedily to the closing brace and a trailing }} is a literal brace: f'{1:}}}' is '1}' and f'{1:>3{5}}}}' right-aligns 1 in width 5 then appends '}'"""
# }} after a field's closing brace is a literal '}'

# The format spec is matched greedily up to the closing brace; the
# trailing }} is a literal brace appended after formatting.
assert f"{1:}}}" == "1}"
assert f"{1:>3{5}}}}" == ("                                  1" + "}")

print("trailing_double_brace_is_literal_after_format OK")
"###);
    assert_output(&out, r###"trailing_double_brace_is_literal_after_format OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/498/triple_quoted_embeds_both_quote_styles.py`.
#[test]
fn test_gen_behavior_pep_498_triple_quoted_embeds_both_quote_styles() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "498"
# dimension = "behavior"
# case = "triple_quoted_embeds_both_quote_styles"
# subject = "fstring.quoting"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.quoting: a triple-quoted f-string lets a field embed both quote styles: f'''{"eric's"}''' is "eric's" and f'''{'xeric"sy'}''' is 'xeric"sy'"""
# triple-quoted f-strings relax the inner quote constraint

assert f"""{"eric's"}""" == "eric's"
assert f"""{'xeric"sy'}""" == 'xeric"sy'

print("triple_quoted_embeds_both_quote_styles OK")
"###);
    assert_output(&out, r###"triple_quoted_embeds_both_quote_styles OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/498/walrus_binds_and_leaks.py`.
#[test]
fn test_gen_behavior_pep_498_walrus_binds_and_leaks() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "498"
# dimension = "behavior"
# case = "walrus_binds_and_leaks"
# subject = "fstring.expression"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.expression: a walrus inside a field binds and leaks: f'{(z := 10)}' is '10' and z == 10 afterward"""
# the walrus operator works inside a replacement field

assert f"{(z := 10)}" == "10"
assert z == 10

print("walrus_binds_and_leaks OK")
"###);
    assert_output(&out, r###"walrus_binds_and_leaks OK
"###);
}
