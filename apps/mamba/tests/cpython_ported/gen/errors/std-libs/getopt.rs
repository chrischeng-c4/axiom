use super::super::super::super::harness::*;

/// Ported from `tests/cpython/errors/std-libs/getopt/ambiguous_long_prefix_raises.py`.
#[test]
fn test_gen_errors_std_libs_getopt_ambiguous_long_prefix_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "getopt"
# dimension = "errors"
# case = "ambiguous_long_prefix_raises"
# subject = "getopt.getopt"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_getopt.py"
# status = "filled"
# ///
"""getopt.getopt: ambiguous_long_prefix_raises (errors)."""
import getopt

_raised = False
try:
    getopt.getopt(['--he'], '', ['help', 'header'])
except getopt.GetoptError:
    _raised = True
assert _raised, "ambiguous_long_prefix_raises: expected getopt.GetoptError"
print("ambiguous_long_prefix_raises OK")
"###);
    assert_output(&out, r###"ambiguous_long_prefix_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/getopt/long_option_unwanted_arg_raises.py`.
#[test]
fn test_gen_errors_std_libs_getopt_long_option_unwanted_arg_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "getopt"
# dimension = "errors"
# case = "long_option_unwanted_arg_raises"
# subject = "getopt.getopt"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_getopt.py"
# status = "filled"
# ///
"""getopt.getopt: long_option_unwanted_arg_raises (errors)."""
import getopt

_raised = False
try:
    getopt.getopt(['--flag=x'], '', ['flag'])
except getopt.GetoptError:
    _raised = True
assert _raised, "long_option_unwanted_arg_raises: expected getopt.GetoptError"
print("long_option_unwanted_arg_raises OK")
"###);
    assert_output(&out, r###"long_option_unwanted_arg_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/getopt/missing_argument_raises.py`.
#[test]
fn test_gen_errors_std_libs_getopt_missing_argument_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "getopt"
# dimension = "errors"
# case = "missing_argument_raises"
# subject = "getopt.getopt"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_getopt.py"
# status = "filled"
# ///
"""getopt.getopt: missing_argument_raises (errors)."""
import getopt

_raised = False
try:
    getopt.getopt(['-b'], 'ab:')
except getopt.GetoptError:
    _raised = True
assert _raised, "missing_argument_raises: expected getopt.GetoptError"
print("missing_argument_raises OK")
"###);
    assert_output(&out, r###"missing_argument_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/getopt/unknown_long_option_raises.py`.
#[test]
fn test_gen_errors_std_libs_getopt_unknown_long_option_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "getopt"
# dimension = "errors"
# case = "unknown_long_option_raises"
# subject = "getopt.getopt"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_getopt.py"
# status = "filled"
# ///
"""getopt.getopt: unknown_long_option_raises (errors)."""
import getopt

_raised = False
try:
    getopt.getopt(['--unknown'], '', ['help'])
except getopt.GetoptError:
    _raised = True
assert _raised, "unknown_long_option_raises: expected getopt.GetoptError"
print("unknown_long_option_raises OK")
"###);
    assert_output(&out, r###"unknown_long_option_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/getopt/unknown_short_option_raises.py`.
#[test]
fn test_gen_errors_std_libs_getopt_unknown_short_option_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "getopt"
# dimension = "errors"
# case = "unknown_short_option_raises"
# subject = "getopt.getopt"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_getopt.py"
# status = "filled"
# ///
"""getopt.getopt: unknown_short_option_raises (errors)."""
import getopt

_raised = False
try:
    getopt.getopt(['-x'], 'ab:')
except getopt.GetoptError:
    _raised = True
assert _raised, "unknown_short_option_raises: expected getopt.GetoptError"
print("unknown_short_option_raises OK")
"###);
    assert_output(&out, r###"unknown_short_option_raises OK
"###);
}
