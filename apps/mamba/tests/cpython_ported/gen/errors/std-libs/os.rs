use super::super::super::super::harness::*;

/// Ported from `tests/cpython/errors/std-libs/os/direntry_not_instantiable.py`.
#[test]
fn test_gen_errors_std_libs_os_direntry_not_instantiable() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "errors"
# case = "direntry_not_instantiable"
# subject = "os.DirEntry"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_os.py"
# status = "filled"
# ///
"""os.DirEntry: direntry_not_instantiable (errors)."""
import os

_raised = False
try:
    os.DirEntry()
except TypeError:
    _raised = True
assert _raised, "direntry_not_instantiable: expected TypeError"
print("direntry_not_instantiable OK")
"###);
    assert_output(&out, r###"direntry_not_instantiable OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/os/execv_empty_argv_raises.py`.
#[test]
fn test_gen_errors_std_libs_os_execv_empty_argv_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "errors"
# case = "execv_empty_argv_raises"
# subject = "os.execv"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_os.py"
# status = "filled"
# ///
"""os.execv: execv_empty_argv_raises (errors)."""
import os

_raised = False
try:
    os.execv('dummy', [])
except ValueError:
    _raised = True
assert _raised, "execv_empty_argv_raises: expected ValueError"
print("execv_empty_argv_raises OK")
"###);
    assert_output(&out, r###"execv_empty_argv_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/os/kill_bad_pid_raises.py`.
#[test]
fn test_gen_errors_std_libs_os_kill_bad_pid_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "errors"
# case = "kill_bad_pid_raises"
# subject = "os.kill"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_os.py"
# status = "filled"
# ///
"""os.kill: kill_bad_pid_raises (errors)."""
import os

_raised = False
try:
    os.kill(99999999, 0)
except OSError:
    _raised = True
assert _raised, "kill_bad_pid_raises: expected OSError"
print("kill_bad_pid_raises OK")
"###);
    assert_output(&out, r###"kill_bad_pid_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/os/remove_missing_file_raises.py`.
#[test]
fn test_gen_errors_std_libs_os_remove_missing_file_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "errors"
# case = "remove_missing_file_raises"
# subject = "os.remove"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_os.py"
# status = "filled"
# ///
"""os.remove: remove_missing_file_raises (errors)."""
import os

_raised = False
try:
    os.remove('/no/such/file_to_remove_xyzzy')
except FileNotFoundError:
    _raised = True
assert _raised, "remove_missing_file_raises: expected FileNotFoundError"
print("remove_missing_file_raises OK")
"###);
    assert_output(&out, r###"remove_missing_file_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/os/rmdir_nonempty_raises_oserror.py`.
#[test]
fn test_gen_errors_std_libs_os_rmdir_nonempty_raises_oserror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "errors"
# case = "rmdir_nonempty_raises_oserror"
# subject = "os.rmdir"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_os.py"
# status = "filled"
# ///
"""os.rmdir: os.rmdir on a directory that still contains a child raises OSError ('Directory not empty'); cleanup leaves no temp tree behind"""
import os
import tempfile

td = tempfile.mkdtemp()
sub = os.path.join(td, "child")
os.mkdir(sub)
raised = False
try:
    os.rmdir(td)
except OSError:
    raised = True
finally:
    os.rmdir(sub)
    os.rmdir(td)
assert raised, "rmdir on a non-empty directory should raise OSError"
print("rmdir_nonempty_raises_oserror OK")
"###);
    assert_output(&out, r###"rmdir_nonempty_raises_oserror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/os/scandir_empty_string_raises.py`.
#[test]
fn test_gen_errors_std_libs_os_scandir_empty_string_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "errors"
# case = "scandir_empty_string_raises"
# subject = "os.scandir"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_os.py"
# status = "filled"
# ///
"""os.scandir: scandir_empty_string_raises (errors)."""
import os

_raised = False
try:
    list(os.scandir(''))
except FileNotFoundError:
    _raised = True
assert _raised, "scandir_empty_string_raises: expected FileNotFoundError"
print("scandir_empty_string_raises OK")
"###);
    assert_output(&out, r###"scandir_empty_string_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/os/stat_missing_file_raises.py`.
#[test]
fn test_gen_errors_std_libs_os_stat_missing_file_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "errors"
# case = "stat_missing_file_raises"
# subject = "os.stat"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_os.py"
# status = "filled"
# ///
"""os.stat: stat_missing_file_raises (errors)."""
import os

_raised = False
try:
    os.stat('/no/such/path/xyzzy')
except FileNotFoundError:
    _raised = True
assert _raised, "stat_missing_file_raises: expected FileNotFoundError"
print("stat_missing_file_raises OK")
"###);
    assert_output(&out, r###"stat_missing_file_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/os/utime_times_and_ns_raises_valueerror.py`.
#[test]
fn test_gen_errors_std_libs_os_utime_times_and_ns_raises_valueerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "errors"
# case = "utime_times_and_ns_raises_valueerror"
# subject = "os.utime"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_os.py"
# status = "filled"
# ///
"""os.utime: os.utime(path, (5, 5), ns=(5, 5)) rejects supplying both a times tuple and ns= at once with ValueError, against a real temp file"""
import os
import tempfile

with tempfile.TemporaryDirectory() as td:
    fpath = os.path.join(td, "f")
    with open(fpath, "w", encoding="utf-8") as f:
        f.write("")
    raised = False
    try:
        os.utime(fpath, (5, 5), ns=(5, 5))
    except ValueError:
        raised = True
    assert raised, "utime with both times and ns= should raise ValueError"
print("utime_times_and_ns_raises_valueerror OK")
"###);
    assert_output(&out, r###"utime_times_and_ns_raises_valueerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/os/write_str_raises_typeerror.py`.
#[test]
fn test_gen_errors_std_libs_os_write_str_raises_typeerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "errors"
# case = "write_str_raises_typeerror"
# subject = "os.write"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_os.py"
# status = "filled"
# ///
"""os.write: write_str_raises_typeerror (errors)."""
import os

_raised = False
try:
    os.write(1, 'beans')
except TypeError:
    _raised = True
assert _raised, "write_str_raises_typeerror: expected TypeError"
print("write_str_raises_typeerror OK")
"###);
    assert_output(&out, r###"write_str_raises_typeerror OK
"###);
}
