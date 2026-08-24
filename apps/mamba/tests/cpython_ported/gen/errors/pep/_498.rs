use super::super::super::super::harness::*;

/// Ported from `tests/cpython/errors/pep/498/bad_format_spec_raises.py`.
#[test]
fn test_gen_errors_pep_498_bad_format_spec_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "498"
# dimension = "errors"
# case = "bad_format_spec_raises"
# subject = "fstring.format_spec"
# kind = "mechanical"
# xfail = "mamba eval() defers parsing past the strict-typing gate; an invalid format code returns None instead of raising ValueError (project_mamba_pep_silent_divergences_2026_05_27, project_mamba_eval_silent_none_cross_type)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.format_spec: bad_format_spec_raises (errors)."""
# an invalid presentation type in the format spec raises ValueError

_raised = False
try:
    eval('f"{1:Q}"')
except ValueError:
    _raised = True
assert _raised, "bad_format_spec_raises: expected ValueError"
print("bad_format_spec_raises OK")
"###);
    assert_output(&out, r###"bad_format_spec_raises OK
"###);
}

/// Ported from `tests/cpython/errors/pep/498/binary_code_on_float_raises.py`.
#[test]
fn test_gen_errors_pep_498_binary_code_on_float_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "498"
# dimension = "errors"
# case = "binary_code_on_float_raises"
# subject = "fstring.format_spec"
# kind = "mechanical"
# xfail = "mamba returns None for a format-code/value-type mismatch (':b' on float) instead of raising ValueError (project_mamba_pep_silent_divergences_2026_05_27)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.format_spec: binary_code_on_float_raises (errors)."""
# 'b' (binary) applied to a float is a ValueError

_raised = False
try:
    '{:b}'.format(1.5)
except ValueError:
    _raised = True
assert _raised, "binary_code_on_float_raises: expected ValueError"
print("binary_code_on_float_raises OK")
"###);
    assert_output(&out, r###"binary_code_on_float_raises OK
"###);
}

/// Ported from `tests/cpython/errors/pep/498/call_non_callable_raises.py`.
#[test]
fn test_gen_errors_pep_498_call_non_callable_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "498"
# dimension = "errors"
# case = "call_non_callable_raises"
# subject = "fstring.expression"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.expression: call_non_callable_raises (errors)."""
# calling a non-callable inside a field raises TypeError

_raised = False
try:
    eval('f"{(1)()}"')
except TypeError:
    _raised = True
assert _raised, "call_non_callable_raises: expected TypeError"
print("call_non_callable_raises OK")
"###);
    assert_output(&out, r###"call_non_callable_raises OK
"###);
}

/// Ported from `tests/cpython/errors/pep/498/comma_underscore_grouping_raises.py`.
#[test]
fn test_gen_errors_pep_498_comma_underscore_grouping_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "498"
# dimension = "errors"
# case = "comma_underscore_grouping_raises"
# subject = "fstring.format_spec"
# kind = "mechanical"
# xfail = "mamba eval() returns None for the mutually-exclusive ',' and '_' grouping options instead of raising ValueError (project_mamba_pep_silent_divergences_2026_05_27)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.format_spec: comma_underscore_grouping_raises (errors)."""
# ',' and '_' grouping options are mutually exclusive (ValueError)

_raised = False
try:
    eval("f'{1:,_}'")
except ValueError:
    _raised = True
assert _raised, "comma_underscore_grouping_raises: expected ValueError"
print("comma_underscore_grouping_raises OK")
"###);
    assert_output(&out, r###"comma_underscore_grouping_raises OK
"###);
}

/// Ported from `tests/cpython/errors/pep/498/decimal_code_on_float_raises.py`.
#[test]
fn test_gen_errors_pep_498_decimal_code_on_float_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "498"
# dimension = "errors"
# case = "decimal_code_on_float_raises"
# subject = "fstring.format_spec"
# kind = "mechanical"
# xfail = "mamba returns None for a format-code/value-type mismatch (':d' on float) instead of raising ValueError (project_mamba_pep_silent_divergences_2026_05_27)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.format_spec: decimal_code_on_float_raises (errors)."""
# 'd' (decimal-integer) applied to a float is a ValueError

_raised = False
try:
    eval("f'{1.5:d}'")
except ValueError:
    _raised = True
assert _raised, "decimal_code_on_float_raises: expected ValueError"
print("decimal_code_on_float_raises OK")
"###);
    assert_output(&out, r###"decimal_code_on_float_raises OK
"###);
}

/// Ported from `tests/cpython/errors/pep/498/decimal_code_on_str_raises.py`.
#[test]
fn test_gen_errors_pep_498_decimal_code_on_str_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "498"
# dimension = "errors"
# case = "decimal_code_on_str_raises"
# subject = "fstring.format_spec"
# kind = "mechanical"
# xfail = "mamba returns None for a format-code/value-type mismatch (':d' on str) instead of raising ValueError (project_mamba_pep_silent_divergences_2026_05_27)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.format_spec: decimal_code_on_str_raises (errors)."""
# 'd' (decimal-integer) applied to a str is a ValueError

_raised = False
try:
    '{:d}'.format('x')
except ValueError:
    _raised = True
assert _raised, "decimal_code_on_str_raises: expected ValueError"
print("decimal_code_on_str_raises OK")
"###);
    assert_output(&out, r###"decimal_code_on_str_raises OK
"###);
}

/// Ported from `tests/cpython/errors/pep/498/div_zero_propagates.py`.
#[test]
fn test_gen_errors_pep_498_div_zero_propagates() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "498"
# dimension = "errors"
# case = "div_zero_propagates"
# subject = "fstring.expression"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.expression: div_zero_propagates (errors)."""
# expression failures inside a field propagate (builtin eval)

_raised = False
try:
    eval('f"{1/0}"')
except ZeroDivisionError:
    _raised = True
assert _raised, "div_zero_propagates: expected ZeroDivisionError"
print("div_zero_propagates OK")
"###);
    assert_output(&out, r###"div_zero_propagates OK
"###);
}

/// Ported from `tests/cpython/errors/pep/498/empty_expression_raises.py`.
#[test]
fn test_gen_errors_pep_498_empty_expression_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "498"
# dimension = "errors"
# case = "empty_expression_raises"
# subject = "fstring.syntax"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.syntax: empty_expression_raises (errors)."""
# an empty replacement field {} is a SyntaxError

_raised = False
try:
    eval('f"{}"')
except SyntaxError:
    _raised = True
assert _raised, "empty_expression_raises: expected SyntaxError"
print("empty_expression_raises OK")
"###);
    assert_output(&out, r###"empty_expression_raises OK
"###);
}

/// Ported from `tests/cpython/errors/pep/498/float_code_on_str_raises.py`.
#[test]
fn test_gen_errors_pep_498_float_code_on_str_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "498"
# dimension = "errors"
# case = "float_code_on_str_raises"
# subject = "fstring.format_spec"
# kind = "mechanical"
# xfail = "mamba returns None for a format-code/value-type mismatch ('.2f' on str) instead of raising ValueError (project_mamba_pep_silent_divergences_2026_05_27)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.format_spec: float_code_on_str_raises (errors)."""
# '.2f' applied to a str is a ValueError

_raised = False
try:
    '{:.2f}'.format('x')
except ValueError:
    _raised = True
assert _raised, "float_code_on_str_raises: expected ValueError"
print("float_code_on_str_raises OK")
"###);
    assert_output(&out, r###"float_code_on_str_raises OK
"###);
}

/// Ported from `tests/cpython/errors/pep/498/hex_code_on_str_raises.py`.
#[test]
fn test_gen_errors_pep_498_hex_code_on_str_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "498"
# dimension = "errors"
# case = "hex_code_on_str_raises"
# subject = "fstring.format_spec"
# kind = "mechanical"
# xfail = "mamba returns None for a format-code/value-type mismatch (':x' on str) instead of raising ValueError (project_mamba_pep_silent_divergences_2026_05_27)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.format_spec: hex_code_on_str_raises (errors)."""
# 'x' (hex) applied to a str is a ValueError

_raised = False
try:
    '{:x}'.format('a')
except ValueError:
    _raised = True
assert _raised, "hex_code_on_str_raises: expected ValueError"
print("hex_code_on_str_raises OK")
"###);
    assert_output(&out, r###"hex_code_on_str_raises OK
"###);
}

/// Ported from `tests/cpython/errors/pep/498/literal_eval_rejects_fstring.py`.
#[test]
fn test_gen_errors_pep_498_literal_eval_rejects_fstring() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "498"
# dimension = "errors"
# case = "literal_eval_rejects_fstring"
# subject = "ast.literal_eval"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ast.literal_eval: literal_eval_rejects_fstring (errors)."""
import ast

_raised = False
try:
    ast.literal_eval("f'x'")
except ValueError:
    _raised = True
assert _raised, "literal_eval_rejects_fstring: expected ValueError"
print("literal_eval_rejects_fstring OK")
"###);
    assert_output(&out, r###"literal_eval_rejects_fstring OK
"###);
}

/// Ported from `tests/cpython/errors/pep/498/missing_name_raises.py`.
#[test]
fn test_gen_errors_pep_498_missing_name_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "498"
# dimension = "errors"
# case = "missing_name_raises"
# subject = "fstring.expression"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.expression: missing_name_raises (errors)."""
# an unbound name in a field raises NameError at runtime

_raised = False
try:
    eval('f"v:{undefined_name}"')
except NameError:
    _raised = True
assert _raised, "missing_name_raises: expected NameError"
print("missing_name_raises OK")
"###);
    assert_output(&out, r###"missing_name_raises OK
"###);
}

/// Ported from `tests/cpython/errors/pep/498/starred_expression_raises.py`.
#[test]
fn test_gen_errors_pep_498_starred_expression_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "498"
# dimension = "errors"
# case = "starred_expression_raises"
# subject = "fstring.syntax"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.syntax: starred_expression_raises (errors)."""
# a starred expression is not a valid f-string field (SyntaxError)

_raised = False
try:
    compile("f'{*a}'", '?', 'exec')
except SyntaxError:
    _raised = True
assert _raised, "starred_expression_raises: expected SyntaxError"
print("starred_expression_raises OK")
"###);
    assert_output(&out, r###"starred_expression_raises OK
"###);
}

/// Ported from `tests/cpython/errors/pep/498/string_code_on_int_raises.py`.
#[test]
fn test_gen_errors_pep_498_string_code_on_int_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "498"
# dimension = "errors"
# case = "string_code_on_int_raises"
# subject = "fstring.format_spec"
# kind = "mechanical"
# xfail = "mamba returns None for a format-code/value-type mismatch (':s' on int) instead of raising ValueError (project_mamba_pep_silent_divergences_2026_05_27)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.format_spec: string_code_on_int_raises (errors)."""
# 's' (string) applied to an int is a ValueError

_raised = False
try:
    eval("f'{1:s}'")
except ValueError:
    _raised = True
assert _raised, "string_code_on_int_raises: expected ValueError"
print("string_code_on_int_raises OK")
"###);
    assert_output(&out, r###"string_code_on_int_raises OK
"###);
}

/// Ported from `tests/cpython/errors/pep/498/unclosed_brace_raises.py`.
#[test]
fn test_gen_errors_pep_498_unclosed_brace_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "498"
# dimension = "errors"
# case = "unclosed_brace_raises"
# subject = "fstring.syntax"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.syntax: unclosed_brace_raises (errors)."""
# an unclosed replacement-field brace is a SyntaxError

_raised = False
try:
    eval('f"hello {x"')
except SyntaxError:
    _raised = True
assert _raised, "unclosed_brace_raises: expected SyntaxError"
print("unclosed_brace_raises OK")
"###);
    assert_output(&out, r###"unclosed_brace_raises OK
"###);
}
