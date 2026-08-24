use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/getopt/double_dash_terminates_options.py`.
#[test]
fn test_gen_behavior_std_libs_getopt_double_dash_terminates_options() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "getopt"
# dimension = "behavior"
# case = "double_dash_terminates_options"
# subject = "getopt.getopt"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_getopt.py"
# status = "filled"
# ///
"""getopt.getopt: '--' terminates option processing; everything after it is treated as args"""
import getopt

opts, args = getopt.getopt(['-v', '--', '-h'], 'vh')
assert opts == [('-v', '')], opts
assert args == ['-h'], args
print("double_dash_terminates_options OK")
"###);
    assert_output(&out, r###"double_dash_terminates_options OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/getopt/gnu_permutes_nonoptions.py`.
#[test]
fn test_gen_behavior_std_libs_getopt_gnu_permutes_nonoptions() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "getopt"
# dimension = "behavior"
# case = "gnu_permutes_nonoptions"
# subject = "getopt.gnu_getopt"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_getopt.py"
# status = "filled"
# ///
"""getopt.gnu_getopt: GNU getopt permutes options past intervening non-options, collecting them as args"""
import getopt

opts, args = getopt.gnu_getopt(['-v', 'arg', '-h'], 'vh')
assert opts == [('-v', ''), ('-h', '')], opts
assert args == ['arg'], args
print("gnu_permutes_nonoptions OK")
"###);
    assert_output(&out, r###"gnu_permutes_nonoptions OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/getopt/long_option_flag.py`.
#[test]
fn test_gen_behavior_std_libs_getopt_long_option_flag() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "getopt"
# dimension = "behavior"
# case = "long_option_flag"
# subject = "getopt.getopt"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_getopt.py"
# status = "filled"
# ///
"""getopt.getopt: a long flag '--help' (no '=') parses to [('--help', '')] with empty value"""
import getopt

opts, args = getopt.getopt(['--help'], '', ['help'])
assert opts == [('--help', '')], opts
assert args == [], args
print("long_option_flag OK")
"###);
    assert_output(&out, r###"long_option_flag OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/getopt/long_option_inline_value.py`.
#[test]
fn test_gen_behavior_std_libs_getopt_long_option_inline_value() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "getopt"
# dimension = "behavior"
# case = "long_option_inline_value"
# subject = "getopt.getopt"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_getopt.py"
# status = "filled"
# ///
"""getopt.getopt: a long option with inline '--output=foo' splits name and value -> [('--output', 'foo')]"""
import getopt

opts, args = getopt.getopt(['--output=foo'], '', ['output='])
assert opts == [('--output', 'foo')], opts
assert args == [], args
print("long_option_inline_value OK")
"###);
    assert_output(&out, r###"long_option_inline_value OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/getopt/posix_stops_at_first_nonoption.py`.
#[test]
fn test_gen_behavior_std_libs_getopt_posix_stops_at_first_nonoption() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "getopt"
# dimension = "behavior"
# case = "posix_stops_at_first_nonoption"
# subject = "getopt.getopt"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_getopt.py"
# status = "filled"
# ///
"""getopt.getopt: POSIX getopt stops scanning at the first non-option, leaving the rest as args"""
import getopt

opts, args = getopt.getopt(['-v', 'arg', '-h'], 'vh')
assert opts == [('-v', '')], opts
assert args == ['arg', '-h'], args
print("posix_stops_at_first_nonoption OK")
"###);
    assert_output(&out, r###"posix_stops_at_first_nonoption OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/getopt/short_option_no_arg.py`.
#[test]
fn test_gen_behavior_std_libs_getopt_short_option_no_arg() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "getopt"
# dimension = "behavior"
# case = "short_option_no_arg"
# subject = "getopt.getopt"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_getopt.py"
# status = "filled"
# ///
"""getopt.getopt: a bare short flag '-v' parses to [('-v', '')] with no remaining args"""
import getopt

opts, args = getopt.getopt(['-v'], 'v')
assert opts == [('-v', '')], opts
assert args == [], args
print("short_option_no_arg OK")
"###);
    assert_output(&out, r###"short_option_no_arg OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/getopt/short_option_with_arg.py`.
#[test]
fn test_gen_behavior_std_libs_getopt_short_option_with_arg() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "getopt"
# dimension = "behavior"
# case = "short_option_with_arg"
# subject = "getopt.getopt"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_getopt.py"
# status = "filled"
# ///
"""getopt.getopt: a short option declared 'o:' consumes its following argument -> [('-o', 'foo')]"""
import getopt

opts, args = getopt.getopt(['-o', 'foo'], 'o:')
assert opts == [('-o', 'foo')], opts
assert args == [], args
print("short_option_with_arg OK")
"###);
    assert_output(&out, r###"short_option_with_arg OK
"###);
}
