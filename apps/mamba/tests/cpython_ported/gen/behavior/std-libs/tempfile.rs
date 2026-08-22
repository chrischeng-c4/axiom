use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/tempfile/gettempdir_is_cached.py`.
#[test]
fn test_gen_behavior_std_libs_tempfile_gettempdir_is_cached() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "behavior"
# case = "gettempdir_is_cached"
# subject = "tempfile.gettempdir"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tempfile.py"
# status = "filled"
# ///
"""tempfile.gettempdir: gettempdir() caches its result: repeat calls return the very same string object (is-identity)"""
import tempfile

_a = tempfile.gettempdir()
_b = tempfile.gettempdir()
assert _a is _b, "gettempdir is cached"
print("gettempdir_is_cached OK")
"###);
    assert_output(&out, r###"gettempdir_is_cached OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/tempfile/gettempdir_returns_str_dir.py`.
#[test]
fn test_gen_behavior_std_libs_tempfile_gettempdir_returns_str_dir() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "behavior"
# case = "gettempdir_returns_str_dir"
# subject = "tempfile.gettempdir"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tempfile.py"
# status = "filled"
# ///
"""tempfile.gettempdir: gettempdir() returns a non-empty str that names an existing directory"""
import os
import tempfile

_tmpdir = tempfile.gettempdir()
assert isinstance(_tmpdir, str), f"gettempdir type = {type(_tmpdir)!r}"
assert len(_tmpdir) > 0, "gettempdir non-empty"
assert os.path.isdir(_tmpdir), f"tmpdir is dir: {_tmpdir!r}"
print("gettempdir_returns_str_dir OK")
"###);
    assert_output(&out, r###"gettempdir_returns_str_dir OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/tempfile/gettempdirb_is_bytes_form.py`.
#[test]
fn test_gen_behavior_std_libs_tempfile_gettempdirb_is_bytes_form() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "behavior"
# case = "gettempdirb_is_bytes_form"
# subject = "tempfile.gettempdirb"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tempfile.py"
# status = "filled"
# ///
"""tempfile.gettempdirb: gettempdirb() is the bytes form of gettempdir(): bytes type, and os.fsdecode of it equals the str form"""
import os
import tempfile

_a = tempfile.gettempdir()
_c = tempfile.gettempdirb()
assert isinstance(_c, bytes), f"gettempdirb type = {type(_c)!r}"
assert type(_a) is not type(_c), "str vs bytes are distinct types"
assert _a == os.fsdecode(_c), "decoded bytes form matches str form"
print("gettempdirb_is_bytes_form OK")
"###);
    assert_output(&out, r###"gettempdirb_is_bytes_form OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/tempfile/gettempprefix_nonempty_str_and_bytes.py`.
#[test]
fn test_gen_behavior_std_libs_tempfile_gettempprefix_nonempty_str_and_bytes() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "behavior"
# case = "gettempprefix_nonempty_str_and_bytes"
# subject = "tempfile.gettempprefix"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tempfile.py"
# status = "filled"
# ///
"""tempfile.gettempprefix: gettempprefix() is a non-empty str and gettempprefixb() is the corresponding non-empty bytes"""
import tempfile

_p = tempfile.gettempprefix()
_pb = tempfile.gettempprefixb()
assert isinstance(_p, str) and len(_p) > 0, f"prefix = {_p!r}"
assert isinstance(_pb, bytes) and len(_pb) > 0, f"prefixb = {_pb!r}"
print("gettempprefix_nonempty_str_and_bytes OK")
"###);
    assert_output(&out, r###"gettempprefix_nonempty_str_and_bytes OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/tempfile/infer_return_type_matching_args.py`.
#[test]
fn test_gen_behavior_std_libs_tempfile_infer_return_type_matching_args() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "behavior"
# case = "infer_return_type_matching_args"
# subject = "tempfile._infer_return_type"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tempfile.py"
# status = "filled"
# ///
"""tempfile._infer_return_type: multiple matching arguments keep the shared type: str+str -> str, bytes+bytes -> bytes"""
import tempfile

infer = tempfile._infer_return_type
assert infer("", "") is str, "str + str -> str"
assert infer(b"", b"") is bytes, "bytes + bytes -> bytes"
print("infer_return_type_matching_args OK")
"###);
    assert_output(&out, r###"infer_return_type_matching_args OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/tempfile/infer_return_type_none_is_neutral.py`.
#[test]
fn test_gen_behavior_std_libs_tempfile_infer_return_type_none_is_neutral() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "behavior"
# case = "infer_return_type_none_is_neutral"
# subject = "tempfile._infer_return_type"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tempfile.py"
# status = "filled"
# ///
"""tempfile._infer_return_type: None is neutral and combines with either concrete type: None+str -> str, bytes+None -> bytes, None+None -> str"""
import tempfile

infer = tempfile._infer_return_type
assert infer(None, "") is str, "None + str -> str"
assert infer("", None) is str, "str + None -> str"
assert infer(None, None) is str, "None + None -> str"
assert infer(b"", None) is bytes, "bytes + None -> bytes"
assert infer(None, b"") is bytes, "None + bytes -> bytes"
print("infer_return_type_none_is_neutral OK")
"###);
    assert_output(&out, r###"infer_return_type_none_is_neutral OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/tempfile/infer_return_type_single_arg.py`.
#[test]
fn test_gen_behavior_std_libs_tempfile_infer_return_type_single_arg() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "behavior"
# case = "infer_return_type_single_arg"
# subject = "tempfile._infer_return_type"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tempfile.py"
# status = "filled"
# ///
"""tempfile._infer_return_type: single-argument inference: str -> str, bytes -> bytes, None -> str (the default)"""
import tempfile

infer = tempfile._infer_return_type
assert infer("") is str, "single str -> str"
assert infer(b"") is bytes, "single bytes -> bytes"
assert infer(None) is str, "single None -> str"
print("infer_return_type_single_arg OK")
"###);
    assert_output(&out, r###"infer_return_type_single_arg OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/tempfile/mkdtemp_relative_dir_returns_absolute.py`.
#[test]
fn test_gen_behavior_std_libs_tempfile_mkdtemp_relative_dir_returns_absolute() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "behavior"
# case = "mkdtemp_relative_dir_returns_absolute"
# subject = "tempfile.mkdtemp"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tempfile.py"
# status = "filled"
# ///
"""tempfile.mkdtemp: mkdtemp(dir='.') still returns an absolute path"""
import os
import tempfile

_rel = tempfile.mkdtemp(dir=".")
try:
    assert os.path.isabs(_rel), f"mkdtemp(dir='.') absolute = {_rel!r}"
finally:
    os.rmdir(_rel)
print("mkdtemp_relative_dir_returns_absolute OK")
"###);
    assert_output(&out, r###"mkdtemp_relative_dir_returns_absolute OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/tempfile/mkdtemp_returns_existing_dir.py`.
#[test]
fn test_gen_behavior_std_libs_tempfile_mkdtemp_returns_existing_dir() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "behavior"
# case = "mkdtemp_returns_existing_dir"
# subject = "tempfile.mkdtemp"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tempfile.py"
# status = "filled"
# ///
"""tempfile.mkdtemp: mkdtemp() returns a str path to a directory that exists on disk"""
import os
import tempfile

_dpath = tempfile.mkdtemp()
try:
    assert isinstance(_dpath, str), f"mkdtemp type = {type(_dpath)!r}"
    assert os.path.isdir(_dpath), f"mkdtemp is dir: {_dpath!r}"
finally:
    os.rmdir(_dpath)
print("mkdtemp_returns_existing_dir OK")
"###);
    assert_output(&out, r###"mkdtemp_returns_existing_dir OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/tempfile/mkdtemp_unique_paths.py`.
#[test]
fn test_gen_behavior_std_libs_tempfile_mkdtemp_unique_paths() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "behavior"
# case = "mkdtemp_unique_paths"
# subject = "tempfile.mkdtemp"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tempfile.py"
# status = "filled"
# ///
"""tempfile.mkdtemp: two mkdtemp() calls return two distinct existing directories"""
import os
import tempfile

_d1 = tempfile.mkdtemp()
_d2 = tempfile.mkdtemp()
try:
    assert _d1 != _d2, f"mkdtemp unique: {_d1} vs {_d2}"
    assert os.path.isdir(_d1), "d1 is dir"
    assert os.path.isdir(_d2), "d2 is dir"
finally:
    os.rmdir(_d1); os.rmdir(_d2)
print("mkdtemp_unique_paths OK")
"###);
    assert_output(&out, r###"mkdtemp_unique_paths OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/tempfile/mkstemp_prefix_suffix_in_basename.py`.
#[test]
fn test_gen_behavior_std_libs_tempfile_mkstemp_prefix_suffix_in_basename() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "behavior"
# case = "mkstemp_prefix_suffix_in_basename"
# subject = "tempfile.mkstemp"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tempfile.py"
# status = "filled"
# ///
"""tempfile.mkstemp: mkstemp(prefix=, suffix=) embeds the prefix and suffix in the basename of the returned path"""
import os
import tempfile

_fd, _path = tempfile.mkstemp(suffix=".txt", prefix="myapp_")
try:
    _base = os.path.basename(_path)
    assert _base.endswith(".txt"), f"suffix = {_base!r}"
    assert _base.startswith("myapp_"), f"prefix = {_base!r}"
finally:
    os.close(_fd)
    os.unlink(_path)
print("mkstemp_prefix_suffix_in_basename OK")
"###);
    assert_output(&out, r###"mkstemp_prefix_suffix_in_basename OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/tempfile/mkstemp_returns_fd_and_path.py`.
#[test]
fn test_gen_behavior_std_libs_tempfile_mkstemp_returns_fd_and_path() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "behavior"
# case = "mkstemp_returns_fd_and_path"
# subject = "tempfile.mkstemp"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tempfile.py"
# status = "filled"
# ///
"""tempfile.mkstemp: mkstemp() returns an (int fd, str path) pair and the path exists on disk"""
import os
import tempfile

_fd, _path = tempfile.mkstemp()
try:
    assert isinstance(_fd, int), f"fd type = {type(_fd)!r}"
    assert isinstance(_path, str), f"path type = {type(_path)!r}"
    assert os.path.exists(_path), "mkstemp file exists"
finally:
    os.close(_fd)
    os.unlink(_path)
print("mkstemp_returns_fd_and_path OK")
"###);
    assert_output(&out, r###"mkstemp_returns_fd_and_path OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/tempfile/mkstemp_unique_paths.py`.
#[test]
fn test_gen_behavior_std_libs_tempfile_mkstemp_unique_paths() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "behavior"
# case = "mkstemp_unique_paths"
# subject = "tempfile.mkstemp"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tempfile.py"
# status = "filled"
# ///
"""tempfile.mkstemp: two mkstemp() calls return two distinct paths"""
import os
import tempfile

_fd1, _p1 = tempfile.mkstemp()
_fd2, _p2 = tempfile.mkstemp()
try:
    assert _p1 != _p2, f"mkstemp unique: {_p1} vs {_p2}"
finally:
    os.close(_fd1); os.unlink(_p1)
    os.close(_fd2); os.unlink(_p2)
print("mkstemp_unique_paths OK")
"###);
    assert_output(&out, r###"mkstemp_unique_paths OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/tempfile/named_file_binary_roundtrip.py`.
#[test]
fn test_gen_behavior_std_libs_tempfile_named_file_binary_roundtrip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "behavior"
# case = "named_file_binary_roundtrip"
# subject = "tempfile.NamedTemporaryFile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tempfile.py"
# status = "filled"
# ///
"""tempfile.NamedTemporaryFile: bytes written to a NamedTemporaryFile(delete=False) survive a close and reopen-by-name read"""
import os
import tempfile

with tempfile.NamedTemporaryFile(delete=False) as _ntf:
    _ntf_name = _ntf.name
    _ntf.write(b"\x00\x01\x02\x03")
    _ntf.flush()
try:
    with open(_ntf_name, "rb") as _f:
        _data = _f.read()
    assert _data == b"\x00\x01\x02\x03", f"ntf binary = {_data!r}"
finally:
    os.unlink(_ntf_name)
print("named_file_binary_roundtrip OK")
"###);
    assert_output(&out, r###"named_file_binary_roundtrip OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/tempfile/named_file_close_idempotent.py`.
#[test]
fn test_gen_behavior_std_libs_tempfile_named_file_close_idempotent() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "behavior"
# case = "named_file_close_idempotent"
# subject = "tempfile.NamedTemporaryFile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tempfile.py"
# status = "filled"
# ///
"""tempfile.NamedTemporaryFile: calling close() repeatedly on a NamedTemporaryFile is idempotent (no error on the 2nd/3rd call)"""
import tempfile

g = tempfile.NamedTemporaryFile()
g.write(b"abc\n")
g.close()
g.close()
g.close()
print("named_file_close_idempotent OK")
"###);
    assert_output(&out, r###"named_file_close_idempotent OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/tempfile/named_file_default_delete_on_exit.py`.
#[test]
fn test_gen_behavior_std_libs_tempfile_named_file_default_delete_on_exit() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "behavior"
# case = "named_file_default_delete_on_exit"
# subject = "tempfile.NamedTemporaryFile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tempfile.py"
# status = "filled"
# ///
"""tempfile.NamedTemporaryFile: default delete=True removes the underlying file when the with-block exits, and reusing the closed file raises ValueError"""
import os
import tempfile

with tempfile.NamedTemporaryFile() as auto:
    assert os.path.exists(auto.name), "exists inside with-block"
    auto_name = auto.name
assert not os.path.exists(auto_name), "deleted after with-block"
_reraised = False
try:
    with auto:
        pass
except ValueError:
    _reraised = True
assert _reraised, "reusing the closed file should raise ValueError"
print("named_file_default_delete_on_exit OK")
"###);
    assert_output(&out, r###"named_file_default_delete_on_exit OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/tempfile/named_file_delete_false_survives_close.py`.
#[test]
fn test_gen_behavior_std_libs_tempfile_named_file_delete_false_survives_close() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "behavior"
# case = "named_file_delete_false_survives_close"
# subject = "tempfile.NamedTemporaryFile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tempfile.py"
# status = "filled"
# ///
"""tempfile.NamedTemporaryFile: delete=False keeps the file after close; the caller must unlink it manually"""
import os
import tempfile

with tempfile.TemporaryDirectory() as d:
    with tempfile.NamedTemporaryFile(dir=d, delete=False) as keep:
        keep.write(b"blat")
        keep_name = keep.name
    assert os.path.exists(keep_name), "delete=False keeps the file"
    os.unlink(keep_name)
    assert os.listdir(d) == [], "dir empty after manual unlink"
print("named_file_delete_false_survives_close OK")
"###);
    assert_output(&out, r###"named_file_delete_false_survives_close OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/tempfile/named_file_delete_on_close_false_defers.py`.
#[test]
fn test_gen_behavior_std_libs_tempfile_named_file_delete_on_close_false_defers() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "behavior"
# case = "named_file_delete_on_close_false_defers"
# subject = "tempfile.NamedTemporaryFile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tempfile.py"
# status = "filled"
# ///
"""tempfile.NamedTemporaryFile: 3.12 delete_on_close=False with delete=True: the file persists after an explicit close() and is removed only at with-block exit"""
import os
import tempfile

with tempfile.TemporaryDirectory() as d2:
    name2 = ""
    with tempfile.NamedTemporaryFile(dir=d2, delete=True,
                                     delete_on_close=False) as dc:
        dc.write(b"blat")
        name2 = dc.name
        dc.close()
        assert os.path.exists(name2), "still present after close()"
    assert not os.path.exists(name2), "removed at context-manager exit"
print("named_file_delete_on_close_false_defers OK")
"###);
    assert_output(&out, r###"named_file_delete_on_close_false_defers OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/tempfile/named_file_text_mode_roundtrip.py`.
#[test]
fn test_gen_behavior_std_libs_tempfile_named_file_text_mode_roundtrip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "behavior"
# case = "named_file_text_mode_roundtrip"
# subject = "tempfile.NamedTemporaryFile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tempfile.py"
# status = "filled"
# ///
"""tempfile.NamedTemporaryFile: mode='w' makes NamedTemporaryFile a text file; the written string reads back identically from the named path"""
import os
import tempfile

with tempfile.NamedTemporaryFile(mode="w", suffix=".txt", delete=False) as _ntf:
    _name = _ntf.name
    _ntf.write("text line\n")
try:
    with open(_name) as _f:
        assert _f.read() == "text line\n", "text mode ntf"
finally:
    os.unlink(_name)
print("named_file_text_mode_roundtrip OK")
"###);
    assert_output(&out, r###"named_file_text_mode_roundtrip OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/tempfile/spooled_binary_attrs_flip_on_rollover.py`.
#[test]
fn test_gen_behavior_std_libs_tempfile_spooled_binary_attrs_flip_on_rollover() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "behavior"
# case = "spooled_binary_attrs_flip_on_rollover"
# subject = "tempfile.SpooledTemporaryFile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tempfile.py"
# status = "filled"
# ///
"""tempfile.SpooledTemporaryFile: a binary spool reports mode 'w+b'/name None before rollover and mode 'rb+'/a real name after; text-only attrs (newlines/encoding/errors) raise AttributeError"""
import tempfile

f = tempfile.SpooledTemporaryFile(max_size=10)
f.write(b"x" * 10)
assert not f._rolled, "still in memory at exactly max_size"
assert f.mode == "w+b", f"pre-rollover mode = {f.mode!r}"
assert f.name is None, f"pre-rollover name = {f.name!r}"
for attr in ("newlines", "encoding", "errors"):
    _raised = False
    try:
        getattr(f, attr)
    except AttributeError:
        _raised = True
    assert _raised, f"binary spool should not expose {attr}"
f.write(b"x")  # exceed max_size -> roll to disk
assert f._rolled, "rolled after exceeding max_size"
assert f.mode == "rb+", f"post-rollover mode = {f.mode!r}"
assert f.name is not None, "rolled spool has a real name"
f.close()
print("spooled_binary_attrs_flip_on_rollover OK")
"###);
    assert_output(&out, r###"spooled_binary_attrs_flip_on_rollover OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/tempfile/spooled_close_idempotent.py`.
#[test]
fn test_gen_behavior_std_libs_tempfile_spooled_close_idempotent() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "behavior"
# case = "spooled_close_idempotent"
# subject = "tempfile.SpooledTemporaryFile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tempfile.py"
# status = "filled"
# ///
"""tempfile.SpooledTemporaryFile: close() on a SpooledTemporaryFile is idempotent both before and after rollover"""
import tempfile

for size, label in [(1024, "before"), (1, "after")]:
    c = tempfile.SpooledTemporaryFile(max_size=size)
    c.write(b"abc\n")
    c.close()
    c.close()
    c.close()
print("spooled_close_idempotent OK")
"###);
    assert_output(&out, r###"spooled_close_idempotent OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/tempfile/spooled_context_manager_closes.py`.
#[test]
fn test_gen_behavior_std_libs_tempfile_spooled_context_manager_closes() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "behavior"
# case = "spooled_context_manager_closes"
# subject = "tempfile.SpooledTemporaryFile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tempfile.py"
# status = "filled"
# ///
"""tempfile.SpooledTemporaryFile: the SpooledTemporaryFile context manager closes the file on exit; re-entering the closed spool raises ValueError"""
import tempfile

with tempfile.SpooledTemporaryFile(max_size=1) as cm:
    assert not cm.closed, "open inside with-block"
assert cm.closed, "closed after with-block"
_raised = False
try:
    with cm:
        pass
except ValueError:
    _raised = True
assert _raised, "reusing closed spool should raise ValueError"
print("spooled_context_manager_closes OK")
"###);
    assert_output(&out, r###"spooled_context_manager_closes OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/tempfile/spooled_generic_alias_subscription.py`.
#[test]
fn test_gen_behavior_std_libs_tempfile_spooled_generic_alias_subscription() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "behavior"
# case = "spooled_generic_alias_subscription"
# subject = "tempfile.SpooledTemporaryFile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tempfile.py"
# status = "filled"
# ///
"""tempfile.SpooledTemporaryFile: SpooledTemporaryFile[bytes] produces a types.GenericAlias"""
import types
import tempfile

_alias = tempfile.SpooledTemporaryFile[bytes]
assert isinstance(_alias, types.GenericAlias), f"alias = {type(_alias)!r}"
print("spooled_generic_alias_subscription OK")
"###);
    assert_output(&out, r###"spooled_generic_alias_subscription OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/tempfile/spooled_rolls_over_at_max_size.py`.
#[test]
fn test_gen_behavior_std_libs_tempfile_spooled_rolls_over_at_max_size() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "behavior"
# case = "spooled_rolls_over_at_max_size"
# subject = "tempfile.SpooledTemporaryFile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tempfile.py"
# status = "filled"
# ///
"""tempfile.SpooledTemporaryFile: a SpooledTemporaryFile stays in memory (_rolled False) until a write exceeds max_size, then rolls to disk and its content survives"""
import tempfile

_spooled = tempfile.SpooledTemporaryFile(max_size=10)
_spooled.write(b"short")
assert not _spooled._rolled, "not yet spilled to disk"
_spooled.write(b"x" * 20)  # exceed max_size=10
assert _spooled._rolled, "rolled after exceeding max_size"
_spooled.seek(0)
_read = _spooled.read()
assert _read.startswith(b"short"), f"spooled content = {_read[:5]!r}"
_spooled.close()
print("spooled_rolls_over_at_max_size OK")
"###);
    assert_output(&out, r###"spooled_rolls_over_at_max_size OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/tempfile/spooled_text_attrs_survive_rollover.py`.
#[test]
fn test_gen_behavior_std_libs_tempfile_spooled_text_attrs_survive_rollover() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "behavior"
# case = "spooled_text_attrs_survive_rollover"
# subject = "tempfile.SpooledTemporaryFile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tempfile.py"
# status = "filled"
# ///
"""tempfile.SpooledTemporaryFile: a text spool (mode='w+', encoding='utf-8') round-trips strings and keeps encoding='utf-8'/errors='strict' across rollover"""
import tempfile

t = tempfile.SpooledTemporaryFile(mode="w+", max_size=10, encoding="utf-8")
t.write("abc\n")
t.seek(0)
assert t.read() == "abc\n", "text round-trip before rollover"
assert not t._rolled, "small text stays in memory"
assert t.mode == "w+" and t.name is None
assert t.encoding == "utf-8", f"encoding = {t.encoding!r}"
assert t.errors == "strict", f"errors = {t.errors!r}"
t.write("xyzzy\n" * 4)  # push over max_size
t.seek(0)
assert t.read() == "abc\n" + "xyzzy\n" * 4, "content survives rollover"
assert t._rolled and t.mode == "w+" and t.name is not None
assert t.encoding == "utf-8" and t.errors == "strict"
t.close()
print("spooled_text_attrs_survive_rollover OK")
"###);
    assert_output(&out, r###"spooled_text_attrs_survive_rollover OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/tempfile/spooled_truncate_before_rollover.py`.
#[test]
fn test_gen_behavior_std_libs_tempfile_spooled_truncate_before_rollover() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "behavior"
# case = "spooled_truncate_before_rollover"
# subject = "tempfile.SpooledTemporaryFile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tempfile.py"
# status = "filled"
# ///
"""tempfile.SpooledTemporaryFile: truncate(n) before rollover trims the in-memory buffer in place (still not _rolled)"""
import tempfile

s = tempfile.SpooledTemporaryFile(max_size=10)
s.write(b"abcdefg\n")
s.truncate(4)
assert not s._rolled, "truncate within max_size stays in memory"
assert s._file.getvalue() == b"abcd", f"truncated buffer = {s._file.getvalue()!r}"
s.close()
print("spooled_truncate_before_rollover OK")
"###);
    assert_output(&out, r###"spooled_truncate_before_rollover OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/tempfile/temporary_directory_cleanup_on_exit.py`.
#[test]
fn test_gen_behavior_std_libs_tempfile_temporary_directory_cleanup_on_exit() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "behavior"
# case = "temporary_directory_cleanup_on_exit"
# subject = "tempfile.TemporaryDirectory"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tempfile.py"
# status = "filled"
# ///
"""tempfile.TemporaryDirectory: TemporaryDirectory removes the directory (and files created inside it) when the with-block exits"""
import os
import tempfile

_path = None
with tempfile.TemporaryDirectory() as _d:
    _path = _d
    _fpath = os.path.join(_d, "test.txt")
    with open(_fpath, "w") as _f:
        _f.write("inside tmpdir")
    assert os.path.exists(_fpath), "file inside tmpdir"
assert not os.path.exists(_path), "tmpdir removed after with"
print("temporary_directory_cleanup_on_exit OK")
"###);
    assert_output(&out, r###"temporary_directory_cleanup_on_exit OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/tempfile/temporary_directory_dir_nests_child.py`.
#[test]
fn test_gen_behavior_std_libs_tempfile_temporary_directory_dir_nests_child() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "behavior"
# case = "temporary_directory_dir_nests_child"
# subject = "tempfile.TemporaryDirectory"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tempfile.py"
# status = "filled"
# ///
"""tempfile.TemporaryDirectory: TemporaryDirectory(dir=parent) creates the child directory under the given parent path"""
import os
import tempfile

with tempfile.TemporaryDirectory() as _parent:
    with tempfile.TemporaryDirectory(dir=_parent) as _child:
        assert _child.startswith(_parent), f"child under parent: {_child!r}"
        assert os.path.isdir(_child), "child is dir"
print("temporary_directory_dir_nests_child OK")
"###);
    assert_output(&out, r###"temporary_directory_dir_nests_child OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/tempfile/temporary_directory_path_is_absolute.py`.
#[test]
fn test_gen_behavior_std_libs_tempfile_temporary_directory_path_is_absolute() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "behavior"
# case = "temporary_directory_path_is_absolute"
# subject = "tempfile.TemporaryDirectory"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tempfile.py"
# status = "filled"
# ///
"""tempfile.TemporaryDirectory: the path yielded by TemporaryDirectory() is an existing, absolute directory while the with-block is open"""
import os
import tempfile

with tempfile.TemporaryDirectory() as _d:
    assert os.path.isdir(_d), f"tmpdir exists = {_d!r}"
    assert os.path.isabs(_d), f"tmpdir is absolute = {_d!r}"
assert not os.path.exists(_d), "tmpdir removed after with"
print("temporary_directory_path_is_absolute OK")
"###);
    assert_output(&out, r###"temporary_directory_path_is_absolute OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/tempfile/temporary_file_binary_roundtrip.py`.
#[test]
fn test_gen_behavior_std_libs_tempfile_temporary_file_binary_roundtrip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "behavior"
# case = "temporary_file_binary_roundtrip"
# subject = "tempfile.TemporaryFile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tempfile.py"
# status = "filled"
# ///
"""tempfile.TemporaryFile: TemporaryFile round-trips bytes: write then seek(0) then read returns the same bytes"""
import tempfile

_tempf = tempfile.TemporaryFile()
assert hasattr(_tempf, "read"), "TemporaryFile has read"
assert hasattr(_tempf, "write"), "TemporaryFile has write"
_tempf.write(b"test bytes")
_tempf.seek(0)
assert _tempf.read() == b"test bytes", "TemporaryFile round-trip"
_tempf.close()
print("temporary_file_binary_roundtrip OK")
"###);
    assert_output(&out, r###"temporary_file_binary_roundtrip OK
"###);
}
