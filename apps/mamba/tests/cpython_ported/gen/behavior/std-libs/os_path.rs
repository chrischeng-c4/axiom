use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/os_path/abspath_returns_absolute_path.py`.
#[test]
fn test_gen_behavior_std_libs_os_path_abspath_returns_absolute_path() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os_path"
# dimension = "behavior"
# case = "abspath_returns_absolute_path"
# subject = "os.path.abspath"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_posixpath.py"
# status = "filled"
# ///
"""os.path.abspath: abspath('.') yields a path that isabs() reports as absolute (cwd-dependent, so only the absoluteness is asserted)"""
import os.path

_abs = os.path.abspath(".")
assert os.path.isabs(_abs), f"abspath is absolute = {_abs!r}"

print("abspath_returns_absolute_path OK")
"###);
    assert_output(&out, r###"abspath_returns_absolute_path OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/os_path/basename_last_component.py`.
#[test]
fn test_gen_behavior_std_libs_os_path_basename_last_component() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os_path"
# dimension = "behavior"
# case = "basename_last_component"
# subject = "os.path.basename"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_posixpath.py"
# status = "filled"
# ///
"""os.path.basename: basename returns the final component; '/usr/local/bin/python' -> 'python', 'file.py' -> 'file.py', and a trailing slash -> '' """
import os.path

assert os.path.basename("/usr/local/bin/python") == "python", "basename"
assert os.path.basename("file.py") == "file.py", "basename no dir"
assert os.path.basename("/usr/local/") == "", "basename trailing slash"

print("basename_last_component OK")
"###);
    assert_output(&out, r###"basename_last_component OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/os_path/commonpath_longest_shared_prefix.py`.
#[test]
fn test_gen_behavior_std_libs_os_path_commonpath_longest_shared_prefix() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os_path"
# dimension = "behavior"
# case = "commonpath_longest_shared_prefix"
# subject = "os.path.commonpath"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_posixpath.py"
# status = "filled"
# ///
"""os.path.commonpath: commonpath returns the longest shared directory prefix; commonpath(['/usr/local/bin','/usr/local/lib']) == '/usr/local'"""
import os.path

_cp = os.path.commonpath(["/usr/local/bin", "/usr/local/lib"])
assert _cp == "/usr/local", f"commonpath = {_cp!r}"

print("commonpath_longest_shared_prefix OK")
"###);
    assert_output(&out, r###"commonpath_longest_shared_prefix OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/os_path/dirname_directory_part.py`.
#[test]
fn test_gen_behavior_std_libs_os_path_dirname_directory_part() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os_path"
# dimension = "behavior"
# case = "dirname_directory_part"
# subject = "os.path.dirname"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_posixpath.py"
# status = "filled"
# ///
"""os.path.dirname: dirname returns the directory part; '/usr/local/bin/python' -> '/usr/local/bin' and a bare 'file.py' -> '' """
import os.path

assert os.path.dirname("/usr/local/bin/python") == "/usr/local/bin", "dirname"
assert os.path.dirname("file.py") == "", "dirname no dir"

print("dirname_directory_part OK")
"###);
    assert_output(&out, r###"dirname_directory_part OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/os_path/exists_isfile_isdir_on_real_paths.py`.
#[test]
fn test_gen_behavior_std_libs_os_path_exists_isfile_isdir_on_real_paths() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os_path"
# dimension = "behavior"
# case = "exists_isfile_isdir_on_real_paths"
# subject = "os.path.exists"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_posixpath.py"
# status = "filled"
# ///
"""os.path.exists: inside a TemporaryDirectory, a written file is exists()/isfile() but not isdir(), the directory is isdir() but not isfile(), and a missing name is not exists()"""
import os.path
import tempfile

with tempfile.TemporaryDirectory() as _tmpdir:
    _file = os.path.join(_tmpdir, "test.txt")
    with open(_file, "w") as _f:
        _f.write("hello")
    assert os.path.exists(_file), "file exists"
    assert os.path.isfile(_file), "isfile"
    assert not os.path.isdir(_file), "not isdir for file"
    assert os.path.isdir(_tmpdir), "isdir for dir"
    assert not os.path.isfile(_tmpdir), "not isfile for dir"
    assert not os.path.exists(os.path.join(_tmpdir, "nofile")), "nonexistent"

print("exists_isfile_isdir_on_real_paths OK")
"###);
    assert_output(&out, r###"exists_isfile_isdir_on_real_paths OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/os_path/exists_nonstring_returns_false_no_raise.py`.
#[test]
fn test_gen_behavior_std_libs_os_path_exists_nonstring_returns_false_no_raise() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os_path"
# dimension = "behavior"
# case = "exists_nonstring_returns_false_no_raise"
# subject = "os.path.exists"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_posixpath.py"
# status = "filled"
# ///
"""os.path.exists: exists is forgiving on a non-path argument: exists(123) returns False rather than raising (CPython behavior, NOT an error path)"""
import os.path

# A non-string/path-like argument does not raise; exists() swallows the
# error and reports False (CPython 3.12 behavior).
assert os.path.exists(123) == False, "exists(int) returns False, no raise"

print("exists_nonstring_returns_false_no_raise OK")
"###);
    assert_output(&out, r###"exists_nonstring_returns_false_no_raise OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/os_path/expanduser_tilde_is_absolute.py`.
#[test]
fn test_gen_behavior_std_libs_os_path_expanduser_tilde_is_absolute() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os_path"
# dimension = "behavior"
# case = "expanduser_tilde_is_absolute"
# subject = "os.path.expanduser"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_posixpath.py"
# status = "filled"
# ///
"""os.path.expanduser: expanduser('~') expands the bare tilde to an absolute home directory path (only absoluteness is asserted, the value is environment-dependent)"""
import os.path

_exp = os.path.expanduser("~")
assert os.path.isabs(_exp), f"expanduser ~ is absolute = {_exp!r}"

print("expanduser_tilde_is_absolute OK")
"###);
    assert_output(&out, r###"expanduser_tilde_is_absolute OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/os_path/getsize_returns_byte_count.py`.
#[test]
fn test_gen_behavior_std_libs_os_path_getsize_returns_byte_count() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os_path"
# dimension = "behavior"
# case = "getsize_returns_byte_count"
# subject = "os.path.getsize"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_posixpath.py"
# status = "filled"
# ///
"""os.path.getsize: getsize returns the file's byte length; an 11-byte payload written to a temp file reports getsize == 11"""
import os
import os.path
import tempfile

with tempfile.NamedTemporaryFile(delete=False) as _ntf:
    _ntf.write(b"hello world")
    _ntfname = _ntf.name
try:
    _sz = os.path.getsize(_ntfname)
    assert _sz == 11, f"getsize = {_sz!r}"
finally:
    os.unlink(_ntfname)

print("getsize_returns_byte_count OK")
"###);
    assert_output(&out, r###"getsize_returns_byte_count OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/os_path/isabs_distinguishes_absolute_relative.py`.
#[test]
fn test_gen_behavior_std_libs_os_path_isabs_distinguishes_absolute_relative() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os_path"
# dimension = "behavior"
# case = "isabs_distinguishes_absolute_relative"
# subject = "os.path.isabs"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_posixpath.py"
# status = "filled"
# ///
"""os.path.isabs: isabs is True only for a leading-slash path; isabs('/usr') is True, isabs('relative') and isabs('') are False"""
import os.path

assert os.path.isabs("/usr") == True, "absolute path"
assert os.path.isabs("relative") == False, "relative path"
assert os.path.isabs("") == False, "empty not absolute"

print("isabs_distinguishes_absolute_relative OK")
"###);
    assert_output(&out, r###"isabs_distinguishes_absolute_relative OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/os_path/join_absolute_segment_resets.py`.
#[test]
fn test_gen_behavior_std_libs_os_path_join_absolute_segment_resets() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os_path"
# dimension = "behavior"
# case = "join_absolute_segment_resets"
# subject = "os.path.join"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_posixpath.py"
# status = "filled"
# ///
"""os.path.join: an absolute (leading-slash) segment discards everything before it; join('a','b','/abs','c') == '/abs/c' and join('/root','/other') == '/other'"""
import os.path

assert os.path.join("a", "/abs") == "/abs", "absolute segment resets"
assert os.path.join("a", "b", "/abs", "c") == "/abs/c", "abs resets mid-list"
assert os.path.join("/root", "/other") == "/other", "abs-abs reset"

print("join_absolute_segment_resets OK")
"###);
    assert_output(&out, r###"join_absolute_segment_resets OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/os_path/join_combines_segments.py`.
#[test]
fn test_gen_behavior_std_libs_os_path_join_combines_segments() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os_path"
# dimension = "behavior"
# case = "join_combines_segments"
# subject = "os.path.join"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_posixpath.py"
# status = "filled"
# ///
"""os.path.join: join glues path segments with the POSIX '/' separator across two-, three-, and four-segment inputs"""
import os.path

assert os.path.join("a", "b", "c") == "a/b/c", "join 3 segments"
assert os.path.join("a", "b", "c", "d") == "a/b/c/d", "four-segment join"
assert os.path.join("/usr", "local", "bin") == "/usr/local/bin", "join with root"
assert os.path.join("usr", "local", "bin", "python3") == "usr/local/bin/python3", "toolchain path"

print("join_combines_segments OK")
"###);
    assert_output(&out, r###"join_combines_segments OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/os_path/join_trailing_slash_preserved.py`.
#[test]
fn test_gen_behavior_std_libs_os_path_join_trailing_slash_preserved() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os_path"
# dimension = "behavior"
# case = "join_trailing_slash_preserved"
# subject = "os.path.join"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_posixpath.py"
# status = "filled"
# ///
"""os.path.join: join keeps an empty/trailing-slash final component; join('a','') == 'a/' and join('a','b/') == 'a/b/' while join('a/','b') == 'a/b'"""
import os.path

assert os.path.join("a", "") == "a/", "empty final segment"
assert os.path.join("a", "b/") == "a/b/", "trailing slash preserved"
assert os.path.join("a/", "b") == "a/b", "leading slash in first segment"

print("join_trailing_slash_preserved OK")
"###);
    assert_output(&out, r###"join_trailing_slash_preserved OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/os_path/normpath_collapses_dots_and_slashes.py`.
#[test]
fn test_gen_behavior_std_libs_os_path_normpath_collapses_dots_and_slashes() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os_path"
# dimension = "behavior"
# case = "normpath_collapses_dots_and_slashes"
# subject = "os.path.normpath"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_posixpath.py"
# status = "filled"
# ///
"""os.path.normpath: normpath collapses '//', '.', and '..' segments; '/usr//local/../local/bin/./python' -> '/usr/local/bin/python', './a/../b' -> 'b', 'a/b/c/../../d' -> 'a/d', and '/' is unchanged"""
import os.path

assert os.path.normpath("/usr//local/../local/bin/./python") == "/usr/local/bin/python", "normpath absolute"
assert os.path.normpath("a/b/c/../d") == "a/b/d", "normpath relative"
assert os.path.normpath("/usr//local") == "/usr/local", "double slashes (single-slash prefix)"
assert os.path.normpath("./a/../b") == "b", "dot and dot-dot"
assert os.path.normpath("a/b/c/../../d") == "a/d", "multiple dot-dots"
assert os.path.normpath("/") == "/", "root unchanged"

print("normpath_collapses_dots_and_slashes OK")
"###);
    assert_output(&out, r###"normpath_collapses_dots_and_slashes OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/os_path/realpath_missing_path_does_not_raise.py`.
#[test]
fn test_gen_behavior_std_libs_os_path_realpath_missing_path_does_not_raise() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os_path"
# dimension = "behavior"
# case = "realpath_missing_path_does_not_raise"
# subject = "os.path.realpath"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_posixpath.py"
# status = "filled"
# ///
"""os.path.realpath: realpath on a non-existent path does not raise; it returns the canonicalized path string (here still rooted under '/no')"""
import os.path

_rp = os.path.realpath("/no/such/path")
assert _rp.startswith("/no"), f"realpath missing returns path = {_rp!r}"

print("realpath_missing_path_does_not_raise OK")
"###);
    assert_output(&out, r###"realpath_missing_path_does_not_raise OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/os_path/relpath_child_parent_sibling.py`.
#[test]
fn test_gen_behavior_std_libs_os_path_relpath_child_parent_sibling() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os_path"
# dimension = "behavior"
# case = "relpath_child_parent_sibling"
# subject = "os.path.relpath"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_posixpath.py"
# status = "filled"
# ///
"""os.path.relpath: relpath walks the tree; child '/usr/local/bin' from '/usr/local' is 'bin', parent is '..', and sibling '/usr/lib' from '/usr/local/bin' is '../../lib'"""
import os.path

_rel = os.path.relpath("/usr/local/bin", "/usr/local")
assert _rel == "bin", f"relpath child = {_rel!r}"
_rel2 = os.path.relpath("/usr/local", "/usr/local/bin")
assert _rel2 == "..", f"relpath parent = {_rel2!r}"
_rel3 = os.path.relpath("/usr/lib", "/usr/local/bin")
assert _rel3 == "../../lib", f"relpath sibling = {_rel3!r}"

print("relpath_child_parent_sibling OK")
"###);
    assert_output(&out, r###"relpath_child_parent_sibling OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/os_path/samefile_same_path_and_abspath.py`.
#[test]
fn test_gen_behavior_std_libs_os_path_samefile_same_path_and_abspath() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os_path"
# dimension = "behavior"
# case = "samefile_same_path_and_abspath"
# subject = "os.path.samefile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_posixpath.py"
# status = "filled"
# ///
"""os.path.samefile: samefile is True when both arguments resolve to the same file: a temp file vs itself, and the temp file vs its abspath()"""
import os
import os.path
import tempfile

with tempfile.NamedTemporaryFile(delete=False) as _sf:
    _sfname = _sf.name
try:
    assert os.path.samefile(_sfname, _sfname), "samefile same path"
    _abs = os.path.abspath(_sfname)
    assert os.path.samefile(_sfname, _abs), "samefile abspath"
finally:
    os.unlink(_sfname)

print("samefile_same_path_and_abspath OK")
"###);
    assert_output(&out, r###"samefile_same_path_and_abspath OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/os_path/split_head_tail.py`.
#[test]
fn test_gen_behavior_std_libs_os_path_split_head_tail() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os_path"
# dimension = "behavior"
# case = "split_head_tail"
# subject = "os.path.split"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_posixpath.py"
# status = "filled"
# ///
"""os.path.split: split returns (head, tail); '/usr/local/bin/python' splits to ('/usr/local/bin','python') and a bare 'file.py' to ('','file.py')"""
import os.path

_h, _t = os.path.split("/usr/local/bin/python")
assert _h == "/usr/local/bin", f"split head = {_h!r}"
assert _t == "python", f"split tail = {_t!r}"

_h2, _t2 = os.path.split("file.py")
assert _h2 == "", f"no dir head = {_h2!r}"
assert _t2 == "file.py", f"no dir tail = {_t2!r}"

print("split_head_tail OK")
"###);
    assert_output(&out, r###"split_head_tail OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/os_path/splitext_dotfile_has_no_extension.py`.
#[test]
fn test_gen_behavior_std_libs_os_path_splitext_dotfile_has_no_extension() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os_path"
# dimension = "behavior"
# case = "splitext_dotfile_has_no_extension"
# subject = "os.path.splitext"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_posixpath.py"
# status = "filled"
# ///
"""os.path.splitext: a leading-dot hidden file has no extension; splitext('.hidden') == ('.hidden','')"""
import os.path

_n, _e = os.path.splitext(".hidden")
assert _n == ".hidden", f"dotfile name = {_n!r}"
assert _e == "", f"dotfile ext = {_e!r}"

print("splitext_dotfile_has_no_extension OK")
"###);
    assert_output(&out, r###"splitext_dotfile_has_no_extension OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/os_path/splitext_name_and_extension.py`.
#[test]
fn test_gen_behavior_std_libs_os_path_splitext_name_and_extension() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os_path"
# dimension = "behavior"
# case = "splitext_name_and_extension"
# subject = "os.path.splitext"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_posixpath.py"
# status = "filled"
# ///
"""os.path.splitext: splitext peels the last extension; 'file.py' -> ('file','.py'), 'archive.tar.gz' -> ('archive.tar','.gz'), 'noext' -> ('noext','')"""
import os.path

_name, _ext = os.path.splitext("file.py")
assert _name == "file", f"splitext name = {_name!r}"
assert _ext == ".py", f"splitext ext = {_ext!r}"

_name2, _ext2 = os.path.splitext("archive.tar.gz")
assert _name2 == "archive.tar", f"splitext multiple dots = {_name2!r}"
assert _ext2 == ".gz", f"splitext last ext = {_ext2!r}"

_name3, _ext3 = os.path.splitext("noext")
assert _name3 == "noext", f"no ext name = {_name3!r}"
assert _ext3 == "", f"no ext = {_ext3!r}"

print("splitext_name_and_extension OK")
"###);
    assert_output(&out, r###"splitext_name_and_extension OK
"###);
}
