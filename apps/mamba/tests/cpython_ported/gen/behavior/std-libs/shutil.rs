use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/shutil/copy2_returns_dest_path.py`.
#[test]
fn test_gen_behavior_std_libs_shutil_copy2_returns_dest_path() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shutil"
# dimension = "behavior"
# case = "copy2_returns_dest_path"
# subject = "shutil.copy2"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_shutil.py"
# status = "filled"
# ///
"""shutil.copy2: copy2() returns the destination path string and the destination file exists after the copy"""
import shutil
import tempfile
import os

with tempfile.TemporaryDirectory() as d:
    src = os.path.join(d, "s.txt")
    dst = os.path.join(d, "d.txt")
    with open(src, "w") as f:
        f.write("copy2 test")
    result = shutil.copy2(src, dst)
    assert isinstance(result, str), f"copy2 returns path = {type(result)!r}"
    assert os.path.exists(dst), "copy2 dst exists"

print("copy2_returns_dest_path OK")
"###);
    assert_output(&out, r###"copy2_returns_dest_path OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/shutil/copy_preserves_permissions.py`.
#[test]
fn test_gen_behavior_std_libs_shutil_copy_preserves_permissions() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shutil"
# dimension = "behavior"
# case = "copy_preserves_permissions"
# subject = "shutil.copy"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_shutil.py"
# status = "filled"
# ///
"""shutil.copy: copy() copies content and permission bits; a 0o644 source yields a 0o644 destination (stat.S_IMODE)"""
import shutil
import tempfile
import os
import stat

with tempfile.TemporaryDirectory() as d:
    src = os.path.join(d, "src.txt")
    dst = os.path.join(d, "dst.txt")
    with open(src, "w") as f:
        f.write("perm test")
    os.chmod(src, 0o644)
    shutil.copy(src, dst)
    dst_stat = os.stat(dst)
    assert stat.S_IMODE(dst_stat.st_mode) == 0o644, "permissions copied"

print("copy_preserves_permissions OK")
"###);
    assert_output(&out, r###"copy_preserves_permissions OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/shutil/copyfile_binary_roundtrip.py`.
#[test]
fn test_gen_behavior_std_libs_shutil_copyfile_binary_roundtrip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shutil"
# dimension = "behavior"
# case = "copyfile_binary_roundtrip"
# subject = "shutil.copyfile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_shutil.py"
# status = "filled"
# ///
"""shutil.copyfile: copyfile copies a 256-byte binary file byte-for-byte; the destination reads back identical to the source"""
import shutil
import tempfile
import os

with tempfile.TemporaryDirectory() as d:
    src = os.path.join(d, "src.bin")
    dst = os.path.join(d, "dst.bin")
    content = bytes(range(256))
    with open(src, "wb") as f:
        f.write(content)
    shutil.copyfile(src, dst)
    with open(dst, "rb") as f2:
        assert f2.read() == content, "copyfile binary content"

print("copyfile_binary_roundtrip OK")
"###);
    assert_output(&out, r###"copyfile_binary_roundtrip OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/shutil/copyfileobj_streams.py`.
#[test]
fn test_gen_behavior_std_libs_shutil_copyfileobj_streams() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shutil"
# dimension = "behavior"
# case = "copyfileobj_streams"
# subject = "shutil.copyfileobj"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_shutil.py"
# status = "filled"
# ///
"""shutil.copyfileobj: copyfileobj copies bytes from one file-like object to another; an io.BytesIO source content lands verbatim in the destination BytesIO"""
import shutil
import io

src_io = io.BytesIO(b"stream data")
dst_io = io.BytesIO()
shutil.copyfileobj(src_io, dst_io)
assert dst_io.getvalue() == b"stream data", f"copyfileobj = {dst_io.getvalue()!r}"

print("copyfileobj_streams OK")
"###);
    assert_output(&out, r###"copyfileobj_streams OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/shutil/copytree_copies_tree.py`.
#[test]
fn test_gen_behavior_std_libs_shutil_copytree_copies_tree() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shutil"
# dimension = "behavior"
# case = "copytree_copies_tree"
# subject = "shutil.copytree"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_shutil.py"
# status = "filled"
# ///
"""shutil.copytree: copytree recreates a directory tree (a top-level file plus a subdir file) at the destination with identical content"""
import shutil
import tempfile
import os

with tempfile.TemporaryDirectory() as d:
    src_dir = os.path.join(d, "src_tree")
    dst_dir = os.path.join(d, "dst_tree")
    os.makedirs(os.path.join(src_dir, "sub"))
    with open(os.path.join(src_dir, "a.txt"), "w") as f:
        f.write("a")
    with open(os.path.join(src_dir, "sub", "b.txt"), "w") as f:
        f.write("b")
    shutil.copytree(src_dir, dst_dir)
    assert os.path.isdir(os.path.join(dst_dir, "sub")), "sub dir copied"
    with open(os.path.join(dst_dir, "a.txt")) as f2:
        assert f2.read() == "a", "a.txt copied"
    with open(os.path.join(dst_dir, "sub", "b.txt")) as f3:
        assert f3.read() == "b", "b.txt copied"

print("copytree_copies_tree OK")
"###);
    assert_output(&out, r###"copytree_copies_tree OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/shutil/disk_usage_total_invariant.py`.
#[test]
fn test_gen_behavior_std_libs_shutil_disk_usage_total_invariant() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shutil"
# dimension = "behavior"
# case = "disk_usage_total_invariant"
# subject = "shutil.disk_usage"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_shutil.py"
# status = "filled"
# ///
"""shutil.disk_usage: disk_usage('/') returns a (total, used, free) named tuple with total > 0 and total == used + free"""
import shutil

du = shutil.disk_usage("/")
assert hasattr(du, "total"), "has total"
assert hasattr(du, "used"), "has used"
assert hasattr(du, "free"), "has free"
assert du.total > 0, f"total > 0: {du.total!r}"
assert du.total == du.used + du.free, "total = used + free"

print("disk_usage_total_invariant OK")
"###);
    assert_output(&out, r###"disk_usage_total_invariant OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/shutil/get_terminal_size_env_precedence.py`.
#[test]
fn test_gen_behavior_std_libs_shutil_get_terminal_size_env_precedence() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shutil"
# dimension = "behavior"
# case = "get_terminal_size_env_precedence"
# subject = "shutil.get_terminal_size"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_shutil.py"
# status = "filled"
# ///
"""shutil.get_terminal_size: COLUMNS/LINES env vars take precedence over the real terminal; a malformed value is ignored (result stays >= 0); os.environ is saved and restored around the probe"""
import shutil
import os


def _with_env(changes, fn):
    """Run fn() with os.environ patched by `changes` (None = unset),
    restoring the previous state afterward."""
    saved = {k: os.environ.get(k) for k in changes}
    try:
        for k, v in changes.items():
            if v is None:
                os.environ.pop(k, None)
            else:
                os.environ[k] = v
        return fn()
    finally:
        for k, old in saved.items():
            if old is None:
                os.environ.pop(k, None)
            else:
                os.environ[k] = old


# COLUMNS/LINES env vars win over the real terminal.
size = _with_env({"COLUMNS": "777", "LINES": "888"}, shutil.get_terminal_size)
assert size.columns == 777, f"columns = {size.columns}"
assert size.lines == 888, f"lines = {size.lines}"
assert tuple(size) == (777, 888), f"tuple = {tuple(size)!r}"

# COLUMNS only set -> columns from env, lines from terminal/fallback (>= 0).
s2 = _with_env({"COLUMNS": "123", "LINES": None}, shutil.get_terminal_size)
assert s2.columns == 123, f"columns = {s2.columns}"
assert s2.lines >= 0, f"lines = {s2.lines}"

# A malformed env value is ignored; result is still sane (>= 0).
s3 = _with_env({"COLUMNS": "xxx", "LINES": "yyy"}, shutil.get_terminal_size)
assert s3.columns >= 0 and s3.lines >= 0, f"bad env -> {tuple(s3)!r}"

# With no env vars, the explicit fallback path stays sane (>= 0).
s4 = _with_env({"COLUMNS": None, "LINES": None},
               lambda: shutil.get_terminal_size(fallback=(10, 20)))
assert s4.columns >= 0 and s4.lines >= 0, f"fallback -> {tuple(s4)!r}"

print("get_terminal_size_env_precedence OK")
"###);
    assert_output(&out, r###"get_terminal_size_env_precedence OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/shutil/get_terminal_size_namedtuple_shape.py`.
#[test]
fn test_gen_behavior_std_libs_shutil_get_terminal_size_namedtuple_shape() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shutil"
# dimension = "behavior"
# case = "get_terminal_size_namedtuple_shape"
# subject = "shutil.get_terminal_size"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_shutil.py"
# status = "filled"
# ///
"""shutil.get_terminal_size: get_terminal_size() returns a 2-field named tuple with integer .columns and .lines attributes; tuple(size) round-trips to (columns, lines)"""
import shutil

size = shutil.get_terminal_size()
assert hasattr(size, "columns"), "has columns"
assert hasattr(size, "lines"), "has lines"
assert isinstance(size.columns, int), f"columns type = {type(size.columns)!r}"
assert isinstance(size.lines, int), f"lines type = {type(size.lines)!r}"
# Named tuple of length 2, round-tripping to (columns, lines).
assert len(size) == 2, f"len = {len(size)}"
assert tuple(size) == (size.columns, size.lines), f"tuple = {tuple(size)!r}"

print("get_terminal_size_namedtuple_shape OK")
"###);
    assert_output(&out, r###"get_terminal_size_namedtuple_shape OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/shutil/move_renames_file.py`.
#[test]
fn test_gen_behavior_std_libs_shutil_move_renames_file() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shutil"
# dimension = "behavior"
# case = "move_renames_file"
# subject = "shutil.move"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_shutil.py"
# status = "filled"
# ///
"""shutil.move: move() relocates a file: the source path disappears, the destination appears, and the content is preserved"""
import shutil
import tempfile
import os

with tempfile.TemporaryDirectory() as d:
    src = os.path.join(d, "old.txt")
    dst = os.path.join(d, "new.txt")
    with open(src, "w") as f:
        f.write("moving")
    shutil.move(src, dst)
    assert not os.path.exists(src), "source gone after move"
    assert os.path.exists(dst), "dest exists after move"
    with open(dst) as f2:
        assert f2.read() == "moving", "content preserved"

print("move_renames_file OK")
"###);
    assert_output(&out, r###"move_renames_file OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/shutil/rmtree_removes_nested.py`.
#[test]
fn test_gen_behavior_std_libs_shutil_rmtree_removes_nested() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shutil"
# dimension = "behavior"
# case = "rmtree_removes_nested"
# subject = "shutil.rmtree"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_shutil.py"
# status = "filled"
# ///
"""shutil.rmtree: rmtree removes a deeply nested directory tree (a/b/c with a file) so the top-level directory no longer exists"""
import shutil
import tempfile
import os

with tempfile.TemporaryDirectory() as d:
    sub = os.path.join(d, "a", "b", "c")
    os.makedirs(sub)
    with open(os.path.join(sub, "file.txt"), "w") as f:
        f.write("x")
    root = os.path.join(d, "a")
    shutil.rmtree(root)
    assert not os.path.exists(root), "nested rmtree"

print("rmtree_removes_nested OK")
"###);
    assert_output(&out, r###"rmtree_removes_nested OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/shutil/which_missing_returns_none.py`.
#[test]
fn test_gen_behavior_std_libs_shutil_which_missing_returns_none() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shutil"
# dimension = "behavior"
# case = "which_missing_returns_none"
# subject = "shutil.which"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_shutil.py"
# status = "filled"
# ///
"""shutil.which: which() returns None for a command name that does not exist on PATH"""
import shutil

w = shutil.which("definitely_not_a_real_command_xyz_abc")
assert w is None, f"which nonexistent = {w!r}"

print("which_missing_returns_none OK")
"###);
    assert_output(&out, r###"which_missing_returns_none OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/shutil/which_resolves_existing_executable.py`.
#[test]
fn test_gen_behavior_std_libs_shutil_which_resolves_existing_executable() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shutil"
# dimension = "behavior"
# case = "which_resolves_existing_executable"
# subject = "shutil.which"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_shutil.py"
# status = "filled"
# ///
"""shutil.which: which('ls') resolves an on-PATH executable to an absolute path string (os.path.isabs)"""
import shutil
import os

# 'ls' is on PATH on macOS/Linux; which() resolves it to an absolute path.
ls = shutil.which("ls")
assert isinstance(ls, str), f"which('ls') = {ls!r}"
assert os.path.isabs(ls), f"which returns absolute = {ls!r}"

print("which_resolves_existing_executable OK")
"###);
    assert_output(&out, r###"which_resolves_existing_executable OK
"###);
}
