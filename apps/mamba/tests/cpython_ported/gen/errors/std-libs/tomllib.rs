use super::super::super::super::harness::*;

/// Ported from `tests/cpython/errors/std-libs/tomllib/bad_date_raises.py`.
#[test]
fn test_gen_errors_std_libs_tomllib_bad_date_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tomllib"
# dimension = "errors"
# case = "bad_date_raises"
# subject = "tomllib.loads"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tomllib/test_error.py"
# status = "filled"
# ///
"""tomllib.loads: bad_date_raises (errors)."""
import tomllib

_raised = False
try:
    tomllib.loads('d = 2024-13-01\n')
except tomllib.TOMLDecodeError:
    _raised = True
assert _raised, "bad_date_raises: expected tomllib.TOMLDecodeError"
print("bad_date_raises OK")
"###);
    assert_output(&out, r###"bad_date_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/tomllib/duplicate_key_raises.py`.
#[test]
fn test_gen_errors_std_libs_tomllib_duplicate_key_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tomllib"
# dimension = "errors"
# case = "duplicate_key_raises"
# subject = "tomllib.loads"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tomllib/test_error.py"
# status = "filled"
# ///
"""tomllib.loads: duplicate_key_raises (errors)."""
import tomllib

_raised = False
try:
    tomllib.loads('a = 1\na = 2\n')
except tomllib.TOMLDecodeError:
    _raised = True
assert _raised, "duplicate_key_raises: expected tomllib.TOMLDecodeError"
print("duplicate_key_raises OK")
"###);
    assert_output(&out, r###"duplicate_key_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/tomllib/invalid_parse_float_raises.py`.
#[test]
fn test_gen_errors_std_libs_tomllib_invalid_parse_float_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tomllib"
# dimension = "errors"
# case = "invalid_parse_float_raises"
# subject = "tomllib.loads"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tomllib/test_error.py"
# status = "filled"
# ///
"""tomllib.loads: invalid_parse_float_raises (errors)."""
import tomllib

_raised = False
try:
    tomllib.loads('f=0.1', parse_float=lambda s: {})
except ValueError:
    _raised = True
assert _raised, "invalid_parse_float_raises: expected ValueError"
print("invalid_parse_float_raises OK")
"###);
    assert_output(&out, r###"invalid_parse_float_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/tomllib/invalid_statement_raises.py`.
#[test]
fn test_gen_errors_std_libs_tomllib_invalid_statement_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tomllib"
# dimension = "errors"
# case = "invalid_statement_raises"
# subject = "tomllib.loads"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tomllib/test_error.py"
# status = "filled"
# ///
"""tomllib.loads: invalid_statement_raises (errors)."""
import tomllib

_raised = False
try:
    tomllib.loads('not = a = toml = file')
except tomllib.TOMLDecodeError:
    _raised = True
assert _raised, "invalid_statement_raises: expected tomllib.TOMLDecodeError"
print("invalid_statement_raises OK")
"###);
    assert_output(&out, r###"invalid_statement_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/tomllib/invalid_value_char_raises.py`.
#[test]
fn test_gen_errors_std_libs_tomllib_invalid_value_char_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tomllib"
# dimension = "errors"
# case = "invalid_value_char_raises"
# subject = "tomllib.loads"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tomllib/test_error.py"
# status = "filled"
# ///
"""tomllib.loads: invalid_value_char_raises (errors)."""
import tomllib

_raised = False
try:
    tomllib.loads('key = @invalid')
except tomllib.TOMLDecodeError:
    _raised = True
assert _raised, "invalid_value_char_raises: expected tomllib.TOMLDecodeError"
print("invalid_value_char_raises OK")
"###);
    assert_output(&out, r###"invalid_value_char_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/tomllib/missing_value_raises.py`.
#[test]
fn test_gen_errors_std_libs_tomllib_missing_value_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tomllib"
# dimension = "errors"
# case = "missing_value_raises"
# subject = "tomllib.loads"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tomllib/test_error.py"
# status = "filled"
# ///
"""tomllib.loads: missing_value_raises (errors)."""
import tomllib

_raised = False
try:
    tomllib.loads('\n\nfwfw=')
except tomllib.TOMLDecodeError:
    _raised = True
assert _raised, "missing_value_raises: expected tomllib.TOMLDecodeError"
print("missing_value_raises OK")
"###);
    assert_output(&out, r###"missing_value_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/tomllib/str_file_typeerror.py`.
#[test]
fn test_gen_errors_std_libs_tomllib_str_file_typeerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tomllib"
# dimension = "errors"
# case = "str_file_typeerror"
# subject = "tomllib.load"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tomllib/test_misc.py"
# status = "filled"
# ///
"""tomllib.load: str_file_typeerror (errors)."""
import tomllib
import io

_raised = False
try:
    tomllib.load(io.StringIO('a = 1'))
except TypeError:
    _raised = True
assert _raised, "str_file_typeerror: expected TypeError"
print("str_file_typeerror OK")
"###);
    assert_output(&out, r###"str_file_typeerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/tomllib/unclosed_array_raises.py`.
#[test]
fn test_gen_errors_std_libs_tomllib_unclosed_array_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tomllib"
# dimension = "errors"
# case = "unclosed_array_raises"
# subject = "tomllib.loads"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tomllib/test_error.py"
# status = "filled"
# ///
"""tomllib.loads: unclosed_array_raises (errors)."""
import tomllib

_raised = False
try:
    tomllib.loads('key = [unclosed')
except tomllib.TOMLDecodeError:
    _raised = True
assert _raised, "unclosed_array_raises: expected tomllib.TOMLDecodeError"
print("unclosed_array_raises OK")
"###);
    assert_output(&out, r###"unclosed_array_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/tomllib/unclosed_inline_table_raises.py`.
#[test]
fn test_gen_errors_std_libs_tomllib_unclosed_inline_table_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tomllib"
# dimension = "errors"
# case = "unclosed_inline_table_raises"
# subject = "tomllib.loads"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tomllib/test_error.py"
# status = "filled"
# ///
"""tomllib.loads: unclosed_inline_table_raises (errors)."""
import tomllib

_raised = False
try:
    tomllib.loads('key = {unclosed')
except tomllib.TOMLDecodeError:
    _raised = True
assert _raised, "unclosed_inline_table_raises: expected tomllib.TOMLDecodeError"
print("unclosed_inline_table_raises OK")
"###);
    assert_output(&out, r###"unclosed_inline_table_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/tomllib/unterminated_string_raises.py`.
#[test]
fn test_gen_errors_std_libs_tomllib_unterminated_string_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tomllib"
# dimension = "errors"
# case = "unterminated_string_raises"
# subject = "tomllib.loads"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tomllib/test_error.py"
# status = "filled"
# ///
"""tomllib.loads: unterminated_string_raises (errors)."""
import tomllib

_raised = False
try:
    tomllib.loads('a = "unterminated\n')
except tomllib.TOMLDecodeError:
    _raised = True
assert _raised, "unterminated_string_raises: expected tomllib.TOMLDecodeError"
print("unterminated_string_raises OK")
"###);
    assert_output(&out, r###"unterminated_string_raises OK
"###);
}
