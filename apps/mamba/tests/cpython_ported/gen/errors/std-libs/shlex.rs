use super::super::super::super::harness::*;

/// Ported from `tests/cpython/errors/std-libs/shlex/split_int_raises.py`.
#[test]
fn test_gen_errors_std_libs_shlex_split_int_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shlex"
# dimension = "errors"
# case = "split_int_raises"
# subject = "shlex.split"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_shlex.py"
# status = "filled"
# ///
"""shlex.split: split_int_raises (errors)."""
import shlex

_raised = False
try:
    shlex.split(123)
except AttributeError:
    _raised = True
assert _raised, "split_int_raises: expected AttributeError"
print("split_int_raises OK")
"###);
    assert_output(&out, r###"split_int_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/shlex/split_none_raises.py`.
#[test]
fn test_gen_errors_std_libs_shlex_split_none_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shlex"
# dimension = "errors"
# case = "split_none_raises"
# subject = "shlex.split"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_shlex.py"
# status = "filled"
# ///
"""shlex.split: split_none_raises (errors)."""
import shlex

_raised = False
try:
    shlex.split(None)
except ValueError:
    _raised = True
assert _raised, "split_none_raises: expected ValueError"
print("split_none_raises OK")
"###);
    assert_output(&out, r###"split_none_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/shlex/split_trailing_escape_raises.py`.
#[test]
fn test_gen_errors_std_libs_shlex_split_trailing_escape_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shlex"
# dimension = "errors"
# case = "split_trailing_escape_raises"
# subject = "shlex.split"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_shlex.py"
# status = "filled"
# ///
"""shlex.split: split_trailing_escape_raises (errors)."""
import shlex

_raised = False
try:
    shlex.split('a \\')
except ValueError:
    _raised = True
assert _raised, "split_trailing_escape_raises: expected ValueError"
print("split_trailing_escape_raises OK")
"###);
    assert_output(&out, r###"split_trailing_escape_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/shlex/split_unclosed_double_quote_raises.py`.
#[test]
fn test_gen_errors_std_libs_shlex_split_unclosed_double_quote_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shlex"
# dimension = "errors"
# case = "split_unclosed_double_quote_raises"
# subject = "shlex.split"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_shlex.py"
# status = "filled"
# ///
"""shlex.split: split_unclosed_double_quote_raises (errors)."""
import shlex

_raised = False
try:
    shlex.split('a "unclosed quote')
except ValueError:
    _raised = True
assert _raised, "split_unclosed_double_quote_raises: expected ValueError"
print("split_unclosed_double_quote_raises OK")
"###);
    assert_output(&out, r###"split_unclosed_double_quote_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/shlex/split_unclosed_single_quote_raises.py`.
#[test]
fn test_gen_errors_std_libs_shlex_split_unclosed_single_quote_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shlex"
# dimension = "errors"
# case = "split_unclosed_single_quote_raises"
# subject = "shlex.split"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_shlex.py"
# status = "filled"
# ///
"""shlex.split: split_unclosed_single_quote_raises (errors)."""
import shlex

_raised = False
try:
    shlex.split("a 'unclosed single")
except ValueError:
    _raised = True
assert _raised, "split_unclosed_single_quote_raises: expected ValueError"
print("split_unclosed_single_quote_raises OK")
"###);
    assert_output(&out, r###"split_unclosed_single_quote_raises OK
"###);
}
