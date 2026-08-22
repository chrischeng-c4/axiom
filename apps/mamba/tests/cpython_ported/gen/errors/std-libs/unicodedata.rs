use super::super::super::super::harness::*;

/// Ported from `tests/cpython/errors/std-libs/unicodedata/category_multi_char_raises.py`.
#[test]
fn test_gen_errors_std_libs_unicodedata_category_multi_char_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "errors"
# case = "category_multi_char_raises"
# subject = "unicodedata.category"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unicodedata.py"
# status = "filled"
# ///
"""unicodedata.category: category_multi_char_raises (errors)."""
import unicodedata

_raised = False
try:
    unicodedata.category("xx")
except TypeError:
    _raised = True
assert _raised, "category_multi_char_raises: expected TypeError"
print("category_multi_char_raises OK")
"###);
    assert_output(&out, r###"category_multi_char_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/unicodedata/decimal_non_decimal_raises.py`.
#[test]
fn test_gen_errors_std_libs_unicodedata_decimal_non_decimal_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "errors"
# case = "decimal_non_decimal_raises"
# subject = "unicodedata.decimal"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unicodedata.py"
# status = "filled"
# ///
"""unicodedata.decimal: decimal_non_decimal_raises (errors)."""
import unicodedata

_raised = False
try:
    unicodedata.decimal("A")
except ValueError:
    _raised = True
assert _raised, "decimal_non_decimal_raises: expected ValueError"
print("decimal_non_decimal_raises OK")
"###);
    assert_output(&out, r###"decimal_non_decimal_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/unicodedata/digit_non_digit_raises.py`.
#[test]
fn test_gen_errors_std_libs_unicodedata_digit_non_digit_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "errors"
# case = "digit_non_digit_raises"
# subject = "unicodedata.digit"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unicodedata.py"
# status = "filled"
# ///
"""unicodedata.digit: digit_non_digit_raises (errors)."""
import unicodedata

_raised = False
try:
    unicodedata.digit("A")
except ValueError:
    _raised = True
assert _raised, "digit_non_digit_raises: expected ValueError"
print("digit_non_digit_raises OK")
"###);
    assert_output(&out, r###"digit_non_digit_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/unicodedata/lookup_unknown_name_raises.py`.
#[test]
fn test_gen_errors_std_libs_unicodedata_lookup_unknown_name_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "errors"
# case = "lookup_unknown_name_raises"
# subject = "unicodedata.lookup"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unicodedata.py"
# status = "filled"
# ///
"""unicodedata.lookup: lookup_unknown_name_raises (errors)."""
import unicodedata

_raised = False
try:
    unicodedata.lookup("NO_SUCH_CHARACTER_NAME_XYZZY")
except KeyError:
    _raised = True
assert _raised, "lookup_unknown_name_raises: expected KeyError"
print("lookup_unknown_name_raises OK")
"###);
    assert_output(&out, r###"lookup_unknown_name_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/unicodedata/name_no_name_raises.py`.
#[test]
fn test_gen_errors_std_libs_unicodedata_name_no_name_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "errors"
# case = "name_no_name_raises"
# subject = "unicodedata.name"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unicodedata.py"
# status = "filled"
# ///
"""unicodedata.name: name_no_name_raises (errors)."""
import unicodedata

_raised = False
try:
    unicodedata.name(chr(0))
except ValueError:
    _raised = True
assert _raised, "name_no_name_raises: expected ValueError"
print("name_no_name_raises OK")
"###);
    assert_output(&out, r###"name_no_name_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/unicodedata/numeric_non_numeric_raises.py`.
#[test]
fn test_gen_errors_std_libs_unicodedata_numeric_non_numeric_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "errors"
# case = "numeric_non_numeric_raises"
# subject = "unicodedata.numeric"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unicodedata.py"
# status = "filled"
# ///
"""unicodedata.numeric: numeric_non_numeric_raises (errors)."""
import unicodedata

_raised = False
try:
    unicodedata.numeric("A")
except ValueError:
    _raised = True
assert _raised, "numeric_non_numeric_raises: expected ValueError"
print("numeric_non_numeric_raises OK")
"###);
    assert_output(&out, r###"numeric_non_numeric_raises OK
"###);
}
