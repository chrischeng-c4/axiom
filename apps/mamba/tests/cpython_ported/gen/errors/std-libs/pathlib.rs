use super::super::super::super::harness::*;

/// Ported from `tests/cpython/errors/std-libs/pathlib/bytes_argument_raises.py`.
#[test]
fn test_gen_errors_std_libs_pathlib_bytes_argument_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "errors"
# case = "bytes_argument_raises"
# subject = "pathlib.PurePath"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
"""pathlib.PurePath: bytes_argument_raises (errors)."""
import pathlib

_raised = False
try:
    pathlib.PurePath(b'a')
except TypeError:
    _raised = True
assert _raised, "bytes_argument_raises: expected TypeError"
print("bytes_argument_raises OK")
"###);
    assert_output(&out, r###"bytes_argument_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/pathlib/iterdir_missing_raises.py`.
#[test]
fn test_gen_errors_std_libs_pathlib_iterdir_missing_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "errors"
# case = "iterdir_missing_raises"
# subject = "pathlib.Path.iterdir"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
"""pathlib.Path.iterdir: iterdir_missing_raises (errors)."""
import pathlib

_raised = False
try:
    list(pathlib.Path('/no/such/dir/xyzzy_abc').iterdir())
except FileNotFoundError:
    _raised = True
assert _raised, "iterdir_missing_raises: expected FileNotFoundError"
print("iterdir_missing_raises OK")
"###);
    assert_output(&out, r###"iterdir_missing_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/pathlib/match_empty_pattern_raises.py`.
#[test]
fn test_gen_errors_std_libs_pathlib_match_empty_pattern_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "errors"
# case = "match_empty_pattern_raises"
# subject = "pathlib.PurePosixPath.match"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
"""pathlib.PurePosixPath.match: match_empty_pattern_raises (errors)."""
import pathlib

_raised = False
try:
    pathlib.PurePosixPath('a').match('')
except ValueError:
    _raised = True
assert _raised, "match_empty_pattern_raises: expected ValueError"
print("match_empty_pattern_raises OK")
"###);
    assert_output(&out, r###"match_empty_pattern_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/pathlib/mkdir_missing_parent_raises.py`.
#[test]
fn test_gen_errors_std_libs_pathlib_mkdir_missing_parent_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "errors"
# case = "mkdir_missing_parent_raises"
# subject = "pathlib.Path.mkdir"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
"""pathlib.Path.mkdir: mkdir_missing_parent_raises (errors)."""
import pathlib

_raised = False
try:
    pathlib.Path('/no/such/parent_xyzzy/child').mkdir()
except FileNotFoundError:
    _raised = True
assert _raised, "mkdir_missing_parent_raises: expected FileNotFoundError"
print("mkdir_missing_parent_raises OK")
"###);
    assert_output(&out, r###"mkdir_missing_parent_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/pathlib/parents_out_of_range_raises.py`.
#[test]
fn test_gen_errors_std_libs_pathlib_parents_out_of_range_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "errors"
# case = "parents_out_of_range_raises"
# subject = "pathlib.PurePath.parents"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
"""pathlib.PurePath.parents: parents_out_of_range_raises (errors)."""
import pathlib

_raised = False
try:
    pathlib.PurePath('a/b/c').parents[3]
except IndexError:
    _raised = True
assert _raised, "parents_out_of_range_raises: expected IndexError"
print("parents_out_of_range_raises OK")
"###);
    assert_output(&out, r###"parents_out_of_range_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/pathlib/read_text_missing_raises.py`.
#[test]
fn test_gen_errors_std_libs_pathlib_read_text_missing_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "errors"
# case = "read_text_missing_raises"
# subject = "pathlib.Path.read_text"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
"""pathlib.Path.read_text: read_text_missing_raises (errors)."""
import pathlib

_raised = False
try:
    pathlib.Path('/no/such/file/xyzzy_abc').read_text()
except FileNotFoundError:
    _raised = True
assert _raised, "read_text_missing_raises: expected FileNotFoundError"
print("read_text_missing_raises OK")
"###);
    assert_output(&out, r###"read_text_missing_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/pathlib/relative_as_uri_raises.py`.
#[test]
fn test_gen_errors_std_libs_pathlib_relative_as_uri_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "errors"
# case = "relative_as_uri_raises"
# subject = "pathlib.PurePosixPath.as_uri"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
"""pathlib.PurePosixPath.as_uri: relative_as_uri_raises (errors)."""
import pathlib

_raised = False
try:
    pathlib.PurePosixPath('a').as_uri()
except ValueError:
    _raised = True
assert _raised, "relative_as_uri_raises: expected ValueError"
print("relative_as_uri_raises OK")
"###);
    assert_output(&out, r###"relative_as_uri_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/pathlib/resolve_strict_missing_raises.py`.
#[test]
fn test_gen_errors_std_libs_pathlib_resolve_strict_missing_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "errors"
# case = "resolve_strict_missing_raises"
# subject = "pathlib.Path.resolve"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
"""pathlib.Path.resolve: resolve_strict_missing_raises (errors)."""
import pathlib

_raised = False
try:
    pathlib.Path('/no/such/path/to/resolve_strict').resolve(strict=True)
except FileNotFoundError:
    _raised = True
assert _raised, "resolve_strict_missing_raises: expected FileNotFoundError"
print("resolve_strict_missing_raises OK")
"###);
    assert_output(&out, r###"resolve_strict_missing_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/pathlib/unlink_missing_raises.py`.
#[test]
fn test_gen_errors_std_libs_pathlib_unlink_missing_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "errors"
# case = "unlink_missing_raises"
# subject = "pathlib.Path.unlink"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
"""pathlib.Path.unlink: unlink_missing_raises (errors)."""
import pathlib

_raised = False
try:
    pathlib.Path('/no/such/file_to_unlink').unlink()
except FileNotFoundError:
    _raised = True
assert _raised, "unlink_missing_raises: expected FileNotFoundError"
print("unlink_missing_raises OK")
"###);
    assert_output(&out, r###"unlink_missing_raises OK
"###);
}
