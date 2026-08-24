use super::super::super::super::harness::*;

/// Ported from `tests/cpython/errors/std-libs/shutil/copy_missing_source_raises.py`.
#[test]
fn test_gen_errors_std_libs_shutil_copy_missing_source_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shutil"
# dimension = "errors"
# case = "copy_missing_source_raises"
# subject = "shutil.copy"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_shutil.py"
# status = "filled"
# ///
"""shutil.copy: copy() of a non-existent source path raises FileNotFoundError (set up in a TemporaryDirectory)"""
import shutil
import tempfile
import os

_raised = False
with tempfile.TemporaryDirectory() as td:
    try:
        shutil.copy(os.path.join(td, "nonexistent"), os.path.join(td, "dst"))
    except FileNotFoundError:
        _raised = True
assert _raised, "copy_missing_source_raises: expected FileNotFoundError"
print("copy_missing_source_raises OK")
"###);
    assert_output(&out, r###"copy_missing_source_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/shutil/copyfile_same_file_symlink_raises.py`.
#[test]
fn test_gen_errors_std_libs_shutil_copyfile_same_file_symlink_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shutil"
# dimension = "errors"
# case = "copyfile_same_file_symlink_raises"
# subject = "shutil.copyfile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_shutil.py"
# status = "filled"
# ///
"""shutil.copyfile: copyfile() of a file onto a symlink that points back to it raises shutil.SameFileError (src + dst symlink built in a TemporaryDirectory)"""
import shutil
import tempfile
import os

_raised = False
with tempfile.TemporaryDirectory() as td:
    src = os.path.join(td, "cheese")
    dst = os.path.join(td, "shop")
    with open(src, "w", encoding="utf-8") as f:
        f.write("cheddar")
    os.symlink("cheese", dst)  # dst -> cheese == src
    try:
        shutil.copyfile(src, dst)
    except shutil.SameFileError:
        _raised = True
assert _raised, "copyfile_same_file_symlink_raises: expected shutil.SameFileError"
print("copyfile_same_file_symlink_raises OK")
"###);
    assert_output(&out, r###"copyfile_same_file_symlink_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/shutil/copytree_existing_dest_raises.py`.
#[test]
fn test_gen_errors_std_libs_shutil_copytree_existing_dest_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shutil"
# dimension = "errors"
# case = "copytree_existing_dest_raises"
# subject = "shutil.copytree"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_shutil.py"
# status = "filled"
# ///
"""shutil.copytree: copytree() onto an already-existing destination directory raises FileExistsError (both dirs in a TemporaryDirectory)"""
import shutil
import tempfile
import os

_raised = False
with tempfile.TemporaryDirectory() as td:
    a = os.path.join(td, "a")
    b = os.path.join(td, "b")
    os.mkdir(a)
    os.mkdir(b)
    try:
        shutil.copytree(a, b)
    except FileExistsError:
        _raised = True
assert _raised, "copytree_existing_dest_raises: expected FileExistsError"
print("copytree_existing_dest_raises OK")
"###);
    assert_output(&out, r###"copytree_existing_dest_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/shutil/disk_usage_missing_path_raises.py`.
#[test]
fn test_gen_errors_std_libs_shutil_disk_usage_missing_path_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shutil"
# dimension = "errors"
# case = "disk_usage_missing_path_raises"
# subject = "shutil.disk_usage"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_shutil.py"
# status = "filled"
# ///
"""shutil.disk_usage: disk_usage_missing_path_raises (errors)."""
import shutil

_raised = False
try:
    shutil.disk_usage("/no/such/path_for_usage")
except FileNotFoundError:
    _raised = True
assert _raised, "disk_usage_missing_path_raises: expected FileNotFoundError"
print("disk_usage_missing_path_raises OK")
"###);
    assert_output(&out, r###"disk_usage_missing_path_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/shutil/make_archive_bad_format_raises.py`.
#[test]
fn test_gen_errors_std_libs_shutil_make_archive_bad_format_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shutil"
# dimension = "errors"
# case = "make_archive_bad_format_raises"
# subject = "shutil.make_archive"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_shutil.py"
# status = "filled"
# ///
"""shutil.make_archive: make_archive_bad_format_raises (errors)."""
import shutil

_raised = False
try:
    shutil.make_archive("/tmp/mamba_shutil_test_archive", "no_such_format")
except ValueError:
    _raised = True
assert _raised, "make_archive_bad_format_raises: expected ValueError"
print("make_archive_bad_format_raises OK")
"###);
    assert_output(&out, r###"make_archive_bad_format_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/shutil/move_missing_source_raises.py`.
#[test]
fn test_gen_errors_std_libs_shutil_move_missing_source_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shutil"
# dimension = "errors"
# case = "move_missing_source_raises"
# subject = "shutil.move"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_shutil.py"
# status = "filled"
# ///
"""shutil.move: move() of a non-existent source path raises FileNotFoundError (set up in a TemporaryDirectory)"""
import shutil
import tempfile
import os

_raised = False
with tempfile.TemporaryDirectory() as td:
    try:
        shutil.move(os.path.join(td, "nonexistent_src"),
                    os.path.join(td, "dst"))
    except FileNotFoundError:
        _raised = True
assert _raised, "move_missing_source_raises: expected FileNotFoundError"
print("move_missing_source_raises OK")
"###);
    assert_output(&out, r###"move_missing_source_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/shutil/register_archive_format_bad_extra_args_raises.py`.
#[test]
fn test_gen_errors_std_libs_shutil_register_archive_format_bad_extra_args_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shutil"
# dimension = "errors"
# case = "register_archive_format_bad_extra_args_raises"
# subject = "shutil.register_archive_format"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_shutil.py"
# status = "filled"
# ///
"""shutil.register_archive_format: register_archive_format_bad_extra_args_raises (errors)."""
import shutil

_raised = False
try:
    shutil.register_archive_format("mamba_bad_extra", lambda: None, [(1, 2), (1, 2, 3)])
except TypeError:
    _raised = True
assert _raised, "register_archive_format_bad_extra_args_raises: expected TypeError"
print("register_archive_format_bad_extra_args_raises OK")
"###);
    assert_output(&out, r###"register_archive_format_bad_extra_args_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/shutil/rmtree_fifo_raises.py`.
#[test]
fn test_gen_errors_std_libs_shutil_rmtree_fifo_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shutil"
# dimension = "errors"
# case = "rmtree_fifo_raises"
# subject = "shutil.rmtree"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_shutil.py"
# status = "filled"
# ///
"""shutil.rmtree: rmtree() on a named pipe (FIFO) raises NotADirectoryError on POSIX; guarded by hasattr(os, 'mkfifo') so non-POSIX platforms exit 0 via a skip path"""
import shutil
import tempfile
import os

# POSIX-only path: rmtree on a FIFO refuses (it is not a directory). On a
# platform without os.mkfifo the case is structurally skipped but still exits 0.
if hasattr(os, "mkfifo"):
    _raised = False
    with tempfile.TemporaryDirectory() as td:
        fifo = os.path.join(td, "mypipe")
        os.mkfifo(fifo)
        try:
            shutil.rmtree(fifo)
        except NotADirectoryError:
            _raised = True
    assert _raised, "rmtree_fifo_raises: expected NotADirectoryError"

print("rmtree_fifo_raises OK")
"###);
    assert_output(&out, r###"rmtree_fifo_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/shutil/rmtree_ignore_errors_swallows.py`.
#[test]
fn test_gen_errors_std_libs_shutil_rmtree_ignore_errors_swallows() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shutil"
# dimension = "errors"
# case = "rmtree_ignore_errors_swallows"
# subject = "shutil.rmtree"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_shutil.py"
# status = "filled"
# ///
"""shutil.rmtree: rmtree(nonexistent, ignore_errors=True) swallows the FileNotFoundError and returns without raising"""
import shutil

# ignore_errors=True must swallow the would-be FileNotFoundError; reaching the
# assert proves no exception escaped.
shutil.rmtree("/no/such/path_to_rmtree", ignore_errors=True)
assert True, "rmtree(ignore_errors=True) returned without raising"
print("rmtree_ignore_errors_swallows OK")
"###);
    assert_output(&out, r###"rmtree_ignore_errors_swallows OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/shutil/rmtree_missing_path_raises.py`.
#[test]
fn test_gen_errors_std_libs_shutil_rmtree_missing_path_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shutil"
# dimension = "errors"
# case = "rmtree_missing_path_raises"
# subject = "shutil.rmtree"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_shutil.py"
# status = "filled"
# ///
"""shutil.rmtree: rmtree_missing_path_raises (errors)."""
import shutil

_raised = False
try:
    shutil.rmtree("/no/such/path_to_rmtree")
except FileNotFoundError:
    _raised = True
assert _raised, "rmtree_missing_path_raises: expected FileNotFoundError"
print("rmtree_missing_path_raises OK")
"###);
    assert_output(&out, r###"rmtree_missing_path_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/shutil/rmtree_symlink_raises.py`.
#[test]
fn test_gen_errors_std_libs_shutil_rmtree_symlink_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shutil"
# dimension = "errors"
# case = "rmtree_symlink_raises"
# subject = "shutil.rmtree"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_shutil.py"
# status = "filled"
# ///
"""shutil.rmtree: rmtree() refuses to follow a symlink-to-directory and raises OSError instead of deleting the link target (TemporaryDirectory)"""
import shutil
import tempfile
import os

_raised = False
with tempfile.TemporaryDirectory() as td:
    real = os.path.join(td, "realdir")
    link = os.path.join(td, "linkdir")
    os.mkdir(real)
    os.symlink(real, link)
    try:
        shutil.rmtree(link)
    except OSError:
        _raised = True
    # The link target must survive — rmtree refused to follow the link.
    assert os.path.isdir(real), "rmtree must not delete the symlink target"
assert _raised, "rmtree_symlink_raises: expected OSError"
print("rmtree_symlink_raises OK")
"###);
    assert_output(&out, r###"rmtree_symlink_raises OK
"###);
}
