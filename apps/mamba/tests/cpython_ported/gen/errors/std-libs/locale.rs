use super::super::super::super::harness::*;

/// Ported from `tests/cpython/errors/std-libs/locale/atof_non_numeric_raises.py`.
#[test]
fn test_gen_errors_std_libs_locale_atof_non_numeric_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "locale"
# dimension = "errors"
# case = "atof_non_numeric_raises"
# subject = "locale.atof"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_locale.py"
# status = "filled"
# ///
"""locale.atof: atof_non_numeric_raises (errors)."""
import locale

_raised = False
try:
    locale.atof("not a number")
except ValueError:
    _raised = True
assert _raised, "atof_non_numeric_raises: expected ValueError"
print("atof_non_numeric_raises OK")
"###);
    assert_output(&out, r###"atof_non_numeric_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/locale/atoi_non_integer_raises.py`.
#[test]
fn test_gen_errors_std_libs_locale_atoi_non_integer_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "locale"
# dimension = "errors"
# case = "atoi_non_integer_raises"
# subject = "locale.atoi"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_locale.py"
# status = "filled"
# ///
"""locale.atoi: atoi_non_integer_raises (errors)."""
import locale

_raised = False
try:
    locale.atoi("not an int")
except ValueError:
    _raised = True
assert _raised, "atoi_non_integer_raises: expected ValueError"
print("atoi_non_integer_raises OK")
"###);
    assert_output(&out, r###"atoi_non_integer_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/locale/setlocale_unknown_category_int_raises.py`.
#[test]
fn test_gen_errors_std_libs_locale_setlocale_unknown_category_int_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "locale"
# dimension = "errors"
# case = "setlocale_unknown_category_int_raises"
# subject = "locale.setlocale"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_locale.py"
# status = "filled"
# ///
"""locale.setlocale: setlocale_unknown_category_int_raises (errors)."""
import locale

_raised = False
try:
    locale.setlocale(999999, "C")
except locale.Error:
    _raised = True
assert _raised, "setlocale_unknown_category_int_raises: expected locale.Error"
print("setlocale_unknown_category_int_raises OK")
"###);
    assert_output(&out, r###"setlocale_unknown_category_int_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/locale/setlocale_unknown_locale_raises.py`.
#[test]
fn test_gen_errors_std_libs_locale_setlocale_unknown_locale_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "locale"
# dimension = "errors"
# case = "setlocale_unknown_locale_raises"
# subject = "locale.setlocale"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_locale.py"
# status = "filled"
# ///
"""locale.setlocale: setlocale_unknown_locale_raises (errors)."""
import locale

_raised = False
try:
    locale.setlocale(locale.LC_ALL, "no_such_locale_xyzzy")
except locale.Error:
    _raised = True
assert _raised, "setlocale_unknown_locale_raises: expected locale.Error"
print("setlocale_unknown_locale_raises OK")
"###);
    assert_output(&out, r###"setlocale_unknown_locale_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/locale/strcoll_embedded_null_left_raises.py`.
#[test]
fn test_gen_errors_std_libs_locale_strcoll_embedded_null_left_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "locale"
# dimension = "errors"
# case = "strcoll_embedded_null_left_raises"
# subject = "locale.strcoll"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_locale.py"
# status = "filled"
# ///
"""locale.strcoll: strcoll_embedded_null_left_raises (errors)."""
import locale

_raised = False
try:
    locale.strcoll("a\x00", "a")
except ValueError:
    _raised = True
assert _raised, "strcoll_embedded_null_left_raises: expected ValueError"
print("strcoll_embedded_null_left_raises OK")
"###);
    assert_output(&out, r###"strcoll_embedded_null_left_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/locale/strcoll_embedded_null_right_raises.py`.
#[test]
fn test_gen_errors_std_libs_locale_strcoll_embedded_null_right_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "locale"
# dimension = "errors"
# case = "strcoll_embedded_null_right_raises"
# subject = "locale.strcoll"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_locale.py"
# status = "filled"
# ///
"""locale.strcoll: strcoll_embedded_null_right_raises (errors)."""
import locale

_raised = False
try:
    locale.strcoll("a", "a\x00")
except ValueError:
    _raised = True
assert _raised, "strcoll_embedded_null_right_raises: expected ValueError"
print("strcoll_embedded_null_right_raises OK")
"###);
    assert_output(&out, r###"strcoll_embedded_null_right_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/locale/strxfrm_embedded_null_raises.py`.
#[test]
fn test_gen_errors_std_libs_locale_strxfrm_embedded_null_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "locale"
# dimension = "errors"
# case = "strxfrm_embedded_null_raises"
# subject = "locale.strxfrm"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_locale.py"
# status = "filled"
# ///
"""locale.strxfrm: strxfrm_embedded_null_raises (errors)."""
import locale

_raised = False
try:
    locale.strxfrm("a\x00")
except ValueError:
    _raised = True
assert _raised, "strxfrm_embedded_null_raises: expected ValueError"
print("strxfrm_embedded_null_raises OK")
"###);
    assert_output(&out, r###"strxfrm_embedded_null_raises OK
"###);
}
