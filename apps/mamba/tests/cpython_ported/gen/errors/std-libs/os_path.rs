use super::super::super::super::harness::*;

/// Ported from `tests/cpython/errors/std-libs/os_path/commonpath_empty_iterable_raises.py`.
#[test]
fn test_gen_errors_std_libs_os_path_commonpath_empty_iterable_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os_path"
# dimension = "errors"
# case = "commonpath_empty_iterable_raises"
# subject = "os.path.commonpath"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_posixpath.py"
# status = "filled"
# ///
"""os.path.commonpath: commonpath_empty_iterable_raises (errors)."""
import os.path

_raised = False
try:
    os.path.commonpath([])
except ValueError:
    _raised = True
assert _raised, "commonpath_empty_iterable_raises: expected ValueError"
print("commonpath_empty_iterable_raises OK")
"###);
    assert_output(&out, r###"commonpath_empty_iterable_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/os_path/commonpath_mixed_abs_rel_raises.py`.
#[test]
fn test_gen_errors_std_libs_os_path_commonpath_mixed_abs_rel_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os_path"
# dimension = "errors"
# case = "commonpath_mixed_abs_rel_raises"
# subject = "os.path.commonpath"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_posixpath.py"
# status = "filled"
# ///
"""os.path.commonpath: commonpath_mixed_abs_rel_raises (errors)."""
import os.path

_raised = False
try:
    os.path.commonpath(['/abs/path', 'rel/path'])
except ValueError:
    _raised = True
assert _raised, "commonpath_mixed_abs_rel_raises: expected ValueError"
print("commonpath_mixed_abs_rel_raises OK")
"###);
    assert_output(&out, r###"commonpath_mixed_abs_rel_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/os_path/getsize_missing_file_raises.py`.
#[test]
fn test_gen_errors_std_libs_os_path_getsize_missing_file_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os_path"
# dimension = "errors"
# case = "getsize_missing_file_raises"
# subject = "os.path.getsize"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_posixpath.py"
# status = "filled"
# ///
"""os.path.getsize: getsize_missing_file_raises (errors)."""
import os.path

_raised = False
try:
    os.path.getsize('/no/such/file_xyzzy')
except FileNotFoundError:
    _raised = True
assert _raised, "getsize_missing_file_raises: expected FileNotFoundError"
print("getsize_missing_file_raises OK")
"###);
    assert_output(&out, r###"getsize_missing_file_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/os_path/join_int_arg_raises.py`.
#[test]
fn test_gen_errors_std_libs_os_path_join_int_arg_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os_path"
# dimension = "errors"
# case = "join_int_arg_raises"
# subject = "os.path.join"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_posixpath.py"
# status = "filled"
# ///
"""os.path.join: join_int_arg_raises (errors)."""
import os.path

_raised = False
try:
    os.path.join(123, 'x')
except TypeError:
    _raised = True
assert _raised, "join_int_arg_raises: expected TypeError"
print("join_int_arg_raises OK")
"###);
    assert_output(&out, r###"join_int_arg_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/os_path/join_mixed_str_bytes_raises.py`.
#[test]
fn test_gen_errors_std_libs_os_path_join_mixed_str_bytes_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os_path"
# dimension = "errors"
# case = "join_mixed_str_bytes_raises"
# subject = "os.path.join"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_posixpath.py"
# status = "filled"
# ///
"""os.path.join: join_mixed_str_bytes_raises (errors)."""
import os.path

_raised = False
try:
    os.path.join('a', b'b')
except TypeError:
    _raised = True
assert _raised, "join_mixed_str_bytes_raises: expected TypeError"
print("join_mixed_str_bytes_raises OK")
"###);
    assert_output(&out, r###"join_mixed_str_bytes_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/os_path/samefile_missing_files_raises.py`.
#[test]
fn test_gen_errors_std_libs_os_path_samefile_missing_files_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os_path"
# dimension = "errors"
# case = "samefile_missing_files_raises"
# subject = "os.path.samefile"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_posixpath.py"
# status = "filled"
# ///
"""os.path.samefile: samefile_missing_files_raises (errors)."""
import os.path

_raised = False
try:
    os.path.samefile('/no/such/a', '/no/such/b')
except FileNotFoundError:
    _raised = True
assert _raised, "samefile_missing_files_raises: expected FileNotFoundError"
print("samefile_missing_files_raises OK")
"###);
    assert_output(&out, r###"samefile_missing_files_raises OK
"###);
}
