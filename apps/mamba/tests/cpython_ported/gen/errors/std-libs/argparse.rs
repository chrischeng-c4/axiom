use super::super::super::super::harness::*;

/// Ported from `tests/cpython/errors/std-libs/argparse/argument_error_str_is_message.py`.
#[test]
fn test_gen_errors_std_libs_argparse_argument_error_str_is_message() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "argparse"
# dimension = "errors"
# case = "argument_error_str_is_message"
# subject = "argparse.ArgumentError"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""argparse.ArgumentError: ArgumentError(None, msg) with no bound action stringifies to just the message text"""
import argparse

err = argparse.ArgumentError(None, "my error here")
assert str(err) == "my error here", f"argument_error str = {str(err)!r}"
print("argument_error_str_is_message OK")
"###);
    assert_output(&out, r###"argument_error_str_is_message OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/argparse/argument_type_error_becomes_systemexit.py`.
#[test]
fn test_gen_errors_std_libs_argparse_argument_type_error_becomes_systemexit() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "argparse"
# dimension = "errors"
# case = "argument_type_error_becomes_systemexit"
# subject = "argparse.ArgumentTypeError"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""argparse.ArgumentTypeError: a custom type= callable raising ArgumentTypeError is intercepted by argparse and re-raised as SystemExit (status 2), while a valid value parses cleanly"""
import argparse
import contextlib
import io


def positive_int(s: str) -> int:
    v = int(s)
    if v <= 0:
        raise argparse.ArgumentTypeError(f"{s!r} is not positive")
    return v


p = argparse.ArgumentParser(prog="prog")
p.add_argument("--n", type=positive_int)

# Valid value parses cleanly.
ns = p.parse_args(["--n", "5"])
assert ns.n == 5, f"valid value = {ns.n!r}"

# Invalid value: argparse intercepts ArgumentTypeError, raises SystemExit(2).
_code = None
with contextlib.redirect_stderr(io.StringIO()):
    try:
        p.parse_args(["--n", "-3"])
    except SystemExit as e:
        _code = e.code
assert _code == 2, f"ArgumentTypeError exit code = {_code!r}"
print("argument_type_error_becomes_systemexit OK")
"###);
    assert_output(&out, r###"argument_type_error_becomes_systemexit OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/argparse/bad_conflict_handler_raises.py`.
#[test]
fn test_gen_errors_std_libs_argparse_bad_conflict_handler_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "argparse"
# dimension = "errors"
# case = "bad_conflict_handler_raises"
# subject = "argparse.ArgumentParser"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""argparse.ArgumentParser: bad_conflict_handler_raises (errors)."""
import argparse

_raised = False
try:
    argparse.ArgumentParser(conflict_handler='nope')
except ValueError:
    _raised = True
assert _raised, "bad_conflict_handler_raises: expected ValueError"
print("bad_conflict_handler_raises OK")
"###);
    assert_output(&out, r###"bad_conflict_handler_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/argparse/duplicate_option_string_raises.py`.
#[test]
fn test_gen_errors_std_libs_argparse_duplicate_option_string_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "argparse"
# dimension = "errors"
# case = "duplicate_option_string_raises"
# subject = "argparse.ArgumentParser"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""argparse.ArgumentParser: duplicate_option_string_raises (errors)."""
import argparse

_raised = False
try:
    _p = argparse.ArgumentParser(); _p.add_argument('--flag'); _p.add_argument('--flag')
except argparse.ArgumentError:
    _raised = True
assert _raised, "duplicate_option_string_raises: expected argparse.ArgumentError"
print("duplicate_option_string_raises OK")
"###);
    assert_output(&out, r###"duplicate_option_string_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/argparse/duplicate_subparser_name_raises.py`.
#[test]
fn test_gen_errors_std_libs_argparse_duplicate_subparser_name_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "argparse"
# dimension = "errors"
# case = "duplicate_subparser_name_raises"
# subject = "argparse.ArgumentParser"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""argparse.ArgumentParser: duplicate_subparser_name_raises (errors)."""
import argparse

_raised = False
try:
    _p = argparse.ArgumentParser(); _sp = _p.add_subparsers(); _sp.add_parser('build'); _sp.add_parser('build')
except argparse.ArgumentError:
    _raised = True
assert _raised, "duplicate_subparser_name_raises: expected argparse.ArgumentError"
print("duplicate_subparser_name_raises OK")
"###);
    assert_output(&out, r###"duplicate_subparser_name_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/argparse/exit_on_error_false_raises_argument_error.py`.
#[test]
fn test_gen_errors_std_libs_argparse_exit_on_error_false_raises_argument_error() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "argparse"
# dimension = "errors"
# case = "exit_on_error_false_raises_argument_error"
# subject = "argparse.ArgumentParser"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""argparse.ArgumentParser: with exit_on_error=False a parse failure (bad int value) raises ArgumentError instead of printing to stderr and exiting; good args still parse"""
import argparse

p = argparse.ArgumentParser(exit_on_error=False)
p.add_argument("--integers", metavar="N", type=int)

# Good args still parse normally.
ns = p.parse_args(["--integers", "4"])
assert ns.integers == 4, f"exit_on_error good = {ns.integers!r}"

# A bad value raises ArgumentError instead of SystemExit.
_raised = False
try:
    p.parse_args(["--integers", "a"])
except argparse.ArgumentError:
    _raised = True
assert _raised, "exit_on_error=False raises ArgumentError on bad value"
print("exit_on_error_false_raises_argument_error OK")
"###);
    assert_output(&out, r###"exit_on_error_false_raises_argument_error OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/argparse/invalid_choice_exits.py`.
#[test]
fn test_gen_errors_std_libs_argparse_invalid_choice_exits() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "argparse"
# dimension = "errors"
# case = "invalid_choice_exits"
# subject = "argparse.ArgumentParser.parse_args"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""argparse.ArgumentParser.parse_args: parse_args raises SystemExit when a value is not in the declared choices= set, but accepts a valid choice"""
import argparse
import contextlib
import io

p = argparse.ArgumentParser()
p.add_argument("--mode", choices=["fast", "slow"])
ns = p.parse_args(["--mode", "fast"])
assert ns.mode == "fast", f"valid choice = {ns.mode!r}"
_raised = False
with contextlib.redirect_stderr(io.StringIO()):
    try:
        p.parse_args(["--mode", "invalid"])
    except SystemExit:
        _raised = True
assert _raised, "invalid choice raises SystemExit"
print("invalid_choice_exits OK")
"###);
    assert_output(&out, r###"invalid_choice_exits OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/argparse/nargs_plus_empty_exits.py`.
#[test]
fn test_gen_errors_std_libs_argparse_nargs_plus_empty_exits() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "argparse"
# dimension = "errors"
# case = "nargs_plus_empty_exits"
# subject = "argparse.ArgumentParser.parse_args"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""argparse.ArgumentParser.parse_args: a nargs='+' positional with zero supplied values raises SystemExit, but a single value parses"""
import argparse
import contextlib
import io

p = argparse.ArgumentParser()
p.add_argument("items", nargs="+")
ns = p.parse_args(["x"])
assert ns.items == ["x"], f"nargs=+ one = {ns.items!r}"
_raised = False
with contextlib.redirect_stderr(io.StringIO()):
    try:
        p.parse_args([])
    except SystemExit:
        _raised = True
assert _raised, "nargs=+ empty raises SystemExit"
print("nargs_plus_empty_exits OK")
"###);
    assert_output(&out, r###"nargs_plus_empty_exits OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/argparse/required_argument_missing_exits.py`.
#[test]
fn test_gen_errors_std_libs_argparse_required_argument_missing_exits() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "argparse"
# dimension = "errors"
# case = "required_argument_missing_exits"
# subject = "argparse.ArgumentParser.parse_args"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""argparse.ArgumentParser.parse_args: parse_args raises SystemExit when a required=True option is absent from the argument vector"""
import argparse
import contextlib
import io

p = argparse.ArgumentParser()
p.add_argument("--required-arg", required=True)
_raised = False
with contextlib.redirect_stderr(io.StringIO()):
    try:
        p.parse_args([])
    except SystemExit:
        _raised = True
assert _raised, "required argument missing raises SystemExit"
print("required_argument_missing_exits OK")
"###);
    assert_output(&out, r###"required_argument_missing_exits OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/argparse/unknown_option_exits.py`.
#[test]
fn test_gen_errors_std_libs_argparse_unknown_option_exits() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "argparse"
# dimension = "errors"
# case = "unknown_option_exits"
# subject = "argparse.ArgumentParser.parse_args"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""argparse.ArgumentParser.parse_args: parse_args raises SystemExit (status 2) on an unrecognized option, captured with stderr redirected"""
import argparse
import contextlib
import io

p = argparse.ArgumentParser(prog="prog")
p.add_argument("--n", type=int)
_code = None
with contextlib.redirect_stderr(io.StringIO()):
    try:
        p.parse_args(["--unknown"])
    except SystemExit as e:
        _code = e.code
assert _code == 2, f"unknown option exit code = {_code!r}"
print("unknown_option_exits OK")
"###);
    assert_output(&out, r###"unknown_option_exits OK
"###);
}
