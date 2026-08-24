use super::super::super::super::harness::*;

/// Ported from `tests/cpython/errors/std-libs/compileall/ddir_none_no_raise.py`.
#[test]
fn test_gen_errors_std_libs_compileall_ddir_none_no_raise() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "compileall"
# dimension = "errors"
# case = "ddir_none_no_raise"
# subject = "compileall.compile_file"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""compileall.compile_file: compile_file with ddir=None is accepted (does not raise); it only affects the recorded source path and still returns True"""
import compileall
import os
import tempfile

# ddir=None is the default-equivalent: it only influences the source path
# recorded inside the .pyc, never an error condition.
with tempfile.TemporaryDirectory() as d:
    src = os.path.join(d, "m.py")
    with open(src, "w") as f:
        f.write("x = 1\n")
    result = compileall.compile_file(src, ddir=None, quiet=2)
    assert result is True, result
print("ddir_none_no_raise OK")
"###);
    assert_output(&out, r###"ddir_none_no_raise OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/compileall/ddir_with_stripdir_prependdir_raises.py`.
#[test]
fn test_gen_errors_std_libs_compileall_ddir_with_stripdir_prependdir_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "compileall"
# dimension = "errors"
# case = "ddir_with_stripdir_prependdir_raises"
# subject = "compileall.compile_dir"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_compileall.py"
# status = "filled"
# ///
"""compileall.compile_dir: ddir_with_stripdir_prependdir_raises (errors)."""
import compileall
import tempfile

_raised = False
try:
    compileall.compile_dir(tempfile.mkdtemp(), quiet=True, ddir="/bar", stripdir="/foo", prependdir="/bar")
except ValueError:
    _raised = True
assert _raised, "ddir_with_stripdir_prependdir_raises: expected ValueError"
print("ddir_with_stripdir_prependdir_raises OK")
"###);
    assert_output(&out, r###"ddir_with_stripdir_prependdir_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/compileall/missing_dir_returns_true_no_raise.py`.
#[test]
fn test_gen_errors_std_libs_compileall_missing_dir_returns_true_no_raise() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "compileall"
# dimension = "errors"
# case = "missing_dir_returns_true_no_raise"
# subject = "compileall.compile_dir"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""compileall.compile_dir: compile_dir on a non-existent directory does NOT raise; it returns True (no files means no failures)"""
import compileall

# A missing directory yields no work, so compile_dir reports success (True)
# without raising rather than treating the absent tree as a failure.
result = compileall.compile_dir("/no/such/dir", quiet=2)
assert result is True, result
print("missing_dir_returns_true_no_raise OK")
"###);
    assert_output(&out, r###"missing_dir_returns_true_no_raise OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/compileall/missing_file_returns_true_no_raise.py`.
#[test]
fn test_gen_errors_std_libs_compileall_missing_file_returns_true_no_raise() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "compileall"
# dimension = "errors"
# case = "missing_file_returns_true_no_raise"
# subject = "compileall.compile_file"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""compileall.compile_file: compile_file on a non-existent .py path does NOT raise; it reports success (True) because there is nothing to fail at"""
import compileall

# A missing source path is not an error: nothing to compile means nothing to
# fail, so compile_file returns a truthy verdict without raising.
result = compileall.compile_file("/no/such/file.py", quiet=2)
assert result is True, result
print("missing_file_returns_true_no_raise OK")
"###);
    assert_output(&out, r###"missing_file_returns_true_no_raise OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/compileall/optimize_minus_one_no_raise.py`.
#[test]
fn test_gen_errors_std_libs_compileall_optimize_minus_one_no_raise() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "compileall"
# dimension = "errors"
# case = "optimize_minus_one_no_raise"
# subject = "compileall.compile_file"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""compileall.compile_file: optimize=-1 means 'use the interpreter default'; compile_file accepts it without raising and returns True"""
import compileall
import os
import tempfile

# optimize=-1 is the sentinel for "use the running interpreter's optimization
# level" — a documented valid value, not an out-of-range error.
with tempfile.TemporaryDirectory() as d:
    src = os.path.join(d, "opt.py")
    with open(src, "w") as f:
        f.write("x = 1\n")
    result = compileall.compile_file(src, optimize=-1, quiet=2)
    assert result is True, result
print("optimize_minus_one_no_raise OK")
"###);
    assert_output(&out, r###"optimize_minus_one_no_raise OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/compileall/syntax_error_returns_false_no_raise.py`.
#[test]
fn test_gen_errors_std_libs_compileall_syntax_error_returns_false_no_raise() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "compileall"
# dimension = "errors"
# case = "syntax_error_returns_false_no_raise"
# subject = "compileall.compile_file"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_compileall.py"
# status = "filled"
# ///
"""compileall.compile_file: a .py file with a SyntaxError makes compile_file return False rather than raising; the bad source is reported, not propagated"""
import compileall
import os
import tempfile

# A broken source does not propagate the SyntaxError: compile_file catches it,
# reports the failure, and returns a falsy verdict so callers stay in control.
with tempfile.TemporaryDirectory() as d:
    bad = os.path.join(d, "bad.py")
    with open(bad, "w") as f:
        f.write("def f(\n  syntax error here\n")
    result = compileall.compile_file(bad, quiet=2)
    assert result is False, result
print("syntax_error_returns_false_no_raise OK")
"###);
    assert_output(&out, r###"syntax_error_returns_false_no_raise OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/compileall/workers_negative_raises.py`.
#[test]
fn test_gen_errors_std_libs_compileall_workers_negative_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "compileall"
# dimension = "errors"
# case = "workers_negative_raises"
# subject = "compileall.compile_dir"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_compileall.py"
# status = "filled"
# ///
"""compileall.compile_dir: workers_negative_raises (errors)."""
import compileall
import tempfile

_raised = False
try:
    compileall.compile_dir(tempfile.mkdtemp(), workers=-1)
except ValueError:
    _raised = True
assert _raised, "workers_negative_raises: expected ValueError"
print("workers_negative_raises OK")
"###);
    assert_output(&out, r###"workers_negative_raises OK
"###);
}
