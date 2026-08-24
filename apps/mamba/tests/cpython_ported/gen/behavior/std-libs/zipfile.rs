use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/zipfile/append_mode_adds_members.py`.
#[test]
fn test_gen_behavior_std_libs_zipfile_append_mode_adds_members() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile"
# dimension = "behavior"
# case = "append_mode_adds_members"
# subject = "zipfile.ZipFile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""zipfile.ZipFile: reopening an on-disk archive in 'a' mode adds a new member while preserving the existing one"""
import zipfile
import os
import tempfile

with tempfile.TemporaryDirectory() as _td:
    _zippath = os.path.join(_td, "archive.zip")
    with zipfile.ZipFile(_zippath, "w") as _zf:
        _zf.writestr("first.txt", b"first")
    with zipfile.ZipFile(_zippath, "a") as _zf:
        _zf.writestr("second.txt", b"second")
    with zipfile.ZipFile(_zippath, "r") as _zf:
        _names = _zf.namelist()
        assert "first.txt" in _names, f"first in names = {_names!r}"
        assert "second.txt" in _names, "second in names"
        assert _zf.read("first.txt") == b"first", "preserved first content"
        assert _zf.read("second.txt") == b"second", "appended second content"

print("append_mode_adds_members OK")
"###);
    assert_output(&out, r###"append_mode_adds_members OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/zipfile/comment_roundtrip_empty_and_nonempty.py`.
#[test]
fn test_gen_behavior_std_libs_zipfile_comment_roundtrip_empty_and_nonempty() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile"
# dimension = "behavior"
# case = "comment_roundtrip_empty_and_nonempty"
# subject = "zipfile.ZipFile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""zipfile.ZipFile: a bytes comment set on an empty (append) archive and on a non-empty archive both survive a close/reopen"""
import zipfile
import os
import tempfile

# A comment set on an empty (append-mode) archive survives a reopen.
with tempfile.TemporaryDirectory() as _td:
    _zp = os.path.join(_td, "empty.zip")
    with zipfile.ZipFile(_zp, "a", zipfile.ZIP_STORED) as _zf:
        assert not _zf.filelist, "fresh append archive has no entries"
        _zf.comment = b"this is a comment"
    with zipfile.ZipFile(_zp, "r") as _zf:
        assert _zf.comment == b"this is a comment", f"empty-archive comment = {_zf.comment!r}"

# A comment set on a non-empty archive survives a reopen.
with tempfile.TemporaryDirectory() as _td:
    _zp = os.path.join(_td, "data.zip")
    with zipfile.ZipFile(_zp, "w", zipfile.ZIP_STORED) as _zf:
        _zf.writestr("foo.txt", "O, for a Muse of Fire!")
    with zipfile.ZipFile(_zp, "a", zipfile.ZIP_STORED) as _zf:
        assert _zf.filelist, "non-empty archive has entries"
        _zf.comment = b"trailing comment"
    with zipfile.ZipFile(_zp, "r") as _zf:
        assert _zf.comment == b"trailing comment", f"nonempty-archive comment = {_zf.comment!r}"

print("comment_roundtrip_empty_and_nonempty OK")
"###);
    assert_output(&out, r###"comment_roundtrip_empty_and_nonempty OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/zipfile/deflated_compresses_repetitive_data.py`.
#[test]
fn test_gen_behavior_std_libs_zipfile_deflated_compresses_repetitive_data() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile"
# dimension = "behavior"
# case = "deflated_compresses_repetitive_data"
# subject = "zipfile.ZipFile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""zipfile.ZipFile: ZIP_DEFLATED shrinks highly repetitive data so compress_size < file_size while the decompressed content round-trips exactly"""
import zipfile
import io

_data = b"aaaa" * 1000  # highly compressible
_buf = io.BytesIO()
with zipfile.ZipFile(_buf, "w", compression=zipfile.ZIP_DEFLATED) as _zf:
    _zf.writestr("deflated.txt", _data)

_buf.seek(0)
with zipfile.ZipFile(_buf, "r") as _zf:
    _info = _zf.getinfo("deflated.txt")
    assert _info.compress_size < _info.file_size, "DEFLATED: compressed < original"
    assert _zf.read("deflated.txt") == _data, "DEFLATED: decompressed matches"

print("deflated_compresses_repetitive_data OK")
"###);
    assert_output(&out, r###"deflated_compresses_repetitive_data OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/zipfile/extractall_to_disk.py`.
#[test]
fn test_gen_behavior_std_libs_zipfile_extractall_to_disk() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile"
# dimension = "behavior"
# case = "extractall_to_disk"
# subject = "zipfile.ZipFile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""zipfile.ZipFile: extractall writes every member (including a nested subdir) to a temp directory on disk with the correct content"""
import zipfile
import io
import os
import tempfile

with tempfile.TemporaryDirectory() as _tmpdir:
    _buf = io.BytesIO()
    with zipfile.ZipFile(_buf, "w") as _zf:
        _zf.writestr("file1.txt", b"content1")
        _zf.writestr("subdir/file2.txt", b"content2")
    _buf.seek(0)
    with zipfile.ZipFile(_buf, "r") as _zf:
        _zf.extractall(_tmpdir)
    assert os.path.exists(os.path.join(_tmpdir, "file1.txt")), "file1.txt extracted"
    assert os.path.exists(os.path.join(_tmpdir, "subdir", "file2.txt")), \
        "subdir/file2.txt extracted"
    with open(os.path.join(_tmpdir, "file1.txt"), "rb") as _f:
        assert _f.read() == b"content1", "extracted content"

print("extractall_to_disk OK")
"###);
    assert_output(&out, r###"extractall_to_disk OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/zipfile/force_zip64_layout.py`.
#[test]
fn test_gen_behavior_std_libs_zipfile_force_zip64_layout() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile"
# dimension = "behavior"
# case = "force_zip64_layout"
# subject = "zipfile.ZipFile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_zipfile"
# status = "filled"
# ///
"""zipfile.ZipFile: open(..., force_zip64=True) writes the ZIP64 layout even for a 1-byte payload: the fixed header sizes are the 0xFFFFFFFF sentinel, a Zip64 extra field (id 1) carries the true 8-byte sizes, and the reopened member reports extract_version >= ZIP64_VERSION"""
import zipfile
import io
import struct

_data = io.BytesIO()
with zipfile.ZipFile(_data, mode="w", allowZip64=True) as _zf:
    with _zf.open("text.txt", mode="w", force_zip64=True) as _member:
        _member.write(b"_")
_zipdata = _data.getvalue()

(_header, _vers, _os_byte, _flags, _comp, _csize, _usize, _fn_len, _ex_total_len,
 _filename, _ex_id, _ex_len, _ex_usize, _ex_csize, _cd_sig) = struct.unpack(
    "<4sBBHH8xIIHH8shhQQx4s", _zipdata[:63])

assert _header == b"PK\x03\x04", f"local file header sig = {_header!r}"
assert _vers >= zipfile.ZIP64_VERSION, f"version needed = {_vers!r}"
assert _os_byte == 0, f"os byte = {_os_byte!r}"
assert _flags == 0, f"flags = {_flags!r}"
assert _comp == 0, f"compression = {_comp!r}"
# Sizes in the fixed header are the 0xFFFFFFFF sentinel under ZIP64.
assert _csize == 4294967295, f"header compress_size sentinel = {_csize!r}"
assert _usize == 4294967295, f"header file_size sentinel = {_usize!r}"
assert _fn_len == 8, f"filename length = {_fn_len!r}"
assert _filename == b"text.txt", f"filename = {_filename!r}"
# The Zip64 extra field carries the true sizes.
assert _ex_total_len == 20, f"extra total length = {_ex_total_len!r}"
assert _ex_id == 1, f"extra id = {_ex_id!r}"
assert _ex_len == 16, f"extra payload length = {_ex_len!r}"
assert _ex_usize == 1, f"extra file_size = {_ex_usize!r}"
assert _ex_csize == 1, f"extra compress_size = {_ex_csize!r}"
assert _cd_sig == b"PK\x01\x02", f"central dir sig = {_cd_sig!r}"

# Re-reading the archive reports a single member needing ZIP64.
with zipfile.ZipFile(io.BytesIO(_zipdata)) as _z:
    _infos = _z.infolist()
    assert len(_infos) == 1, f"infolist len = {len(_infos)!r}"
    assert _infos[0].extract_version >= zipfile.ZIP64_VERSION, \
        f"extract_version = {_infos[0].extract_version!r}"

print("force_zip64_layout OK")
"###);
    assert_output(&out, r###"force_zip64_layout OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/zipfile/independent_member_positions.py`.
#[test]
fn test_gen_behavior_std_libs_zipfile_independent_member_positions() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile"
# dimension = "behavior"
# case = "independent_member_positions"
# subject = "zipfile.ZipFile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_zipfile"
# status = "filled"
# ///
"""zipfile.ZipFile: two members opened from one archive at the same time keep independent read positions"""
import zipfile
import io

_buf = io.BytesIO()
with zipfile.ZipFile(_buf, "w") as _zf:
    _zf.writestr("a.txt", "123")
    _zf.writestr("b.txt", "456")
_buf.seek(0)
with zipfile.ZipFile(_buf, "r") as _zf:
    with _zf.open("a.txt", "r") as _a, _zf.open("b.txt", "r") as _b:
        assert _a.read(1) == b"1", "a read 1"
        assert _b.seek(1) == 1, "b seek to 1"
        assert _b.read(1) == b"5", "b read after seek"
        assert _a.read(1) == b"2", "a keeps its own position"

print("independent_member_positions OK")
"###);
    assert_output(&out, r###"independent_member_positions OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/zipfile/member_seek_tell_contract.py`.
#[test]
fn test_gen_behavior_std_libs_zipfile_member_seek_tell_contract() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile"
# dimension = "behavior"
# case = "member_seek_tell_contract"
# subject = "zipfile.ZipFile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_zipfile"
# status = "filled"
# ///
"""zipfile.ZipFile: an opened member supports SEEK_SET/SEEK_CUR/SEEK_END seeks with tell() tracking the offset, and read after seeking returns the bytes at that position"""
import zipfile
import io
import os

_txt = b"Where's Bruce?"
_bloc = _txt.find(b"Bruce")

_buf = io.BytesIO()
with zipfile.ZipFile(_buf, "w") as _zf:
    _zf.writestr("foo.txt", _txt)
_buf.seek(0)
with zipfile.ZipFile(_buf, "r") as _zf:
    with _zf.open("foo.txt", "r") as _fp:
        _fp.seek(_bloc, os.SEEK_SET)
        assert _fp.tell() == _bloc, f"after SEEK_SET tell = {_fp.tell()!r}"
        _fp.seek(-_bloc, os.SEEK_CUR)
        assert _fp.tell() == 0, f"after relative back tell = {_fp.tell()!r}"
        _fp.seek(_bloc, os.SEEK_CUR)
        assert _fp.tell() == _bloc, f"after relative fwd tell = {_fp.tell()!r}"
        assert _fp.read(5) == _txt[_bloc:_bloc + 5], "read 5 from seeked pos"
        assert _fp.tell() == _bloc + 5, f"tell after read = {_fp.tell()!r}"
        _fp.seek(0, os.SEEK_END)
        assert _fp.tell() == len(_txt), f"SEEK_END tell = {_fp.tell()!r}"
        _fp.seek(0, os.SEEK_SET)
        assert _fp.tell() == 0, "rewind to 0"

print("member_seek_tell_contract OK")
"###);
    assert_output(&out, r###"member_seek_tell_contract OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/zipfile/multiple_members_independent.py`.
#[test]
fn test_gen_behavior_std_libs_zipfile_multiple_members_independent() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile"
# dimension = "behavior"
# case = "multiple_members_independent"
# subject = "zipfile.ZipFile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""zipfile.ZipFile: several members written to one archive are each retrieved independently with their own content"""
import zipfile
import io

_files = {"a.txt": b"alpha", "b.txt": b"beta", "c.txt": b"gamma"}
_buf = io.BytesIO()
with zipfile.ZipFile(_buf, "w") as _zf:
    for _name, _content in _files.items():
        _zf.writestr(_name, _content)

_buf.seek(0)
with zipfile.ZipFile(_buf, "r") as _zf:
    for _name, _content in _files.items():
        assert _zf.read(_name) == _content, f"content mismatch: {_name}"

print("multiple_members_independent OK")
"###);
    assert_output(&out, r###"multiple_members_independent OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/zipfile/namelist_read_roundtrip.py`.
#[test]
fn test_gen_behavior_std_libs_zipfile_namelist_read_roundtrip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile"
# dimension = "behavior"
# case = "namelist_read_roundtrip"
# subject = "zipfile.ZipFile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""zipfile.ZipFile: writestr two members to an in-memory archive, reopen, and confirm namelist contains both names, read returns the exact bytes, and open() yields a file-like object with the same content"""
import zipfile
import io

_buf = io.BytesIO()
with zipfile.ZipFile(_buf, "w", compression=zipfile.ZIP_STORED) as _zf:
    _zf.writestr("hello.txt", "Hello, World!")
    _zf.writestr("nested/data.txt", "nested content")

_buf.seek(0)
with zipfile.ZipFile(_buf, "r") as _zf:
    _names = _zf.namelist()
    assert isinstance(_names, list), f"namelist type = {type(_names)!r}"
    assert "hello.txt" in _names, f"hello.txt in namelist = {_names!r}"
    assert "nested/data.txt" in _names, "nested in namelist"

    _data = _zf.read("hello.txt")
    assert isinstance(_data, bytes), f"read type = {type(_data)!r}"
    assert _data == b"Hello, World!", f"read data = {_data!r}"

    _infos = _zf.infolist()
    assert isinstance(_infos, list), f"infolist type = {type(_infos)!r}"
    assert len(_infos) == 2, f"infolist len = {len(_infos)!r}"

    with _zf.open("hello.txt") as _fh:
        assert _fh.read() == b"Hello, World!", "open+read content"

print("namelist_read_roundtrip OK")
"###);
    assert_output(&out, r###"namelist_read_roundtrip OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/zipfile/non_ascii_filename_roundtrip.py`.
#[test]
fn test_gen_behavior_std_libs_zipfile_non_ascii_filename_roundtrip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile"
# dimension = "behavior"
# case = "non_ascii_filename_roundtrip"
# subject = "zipfile.ZipFile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""zipfile.ZipFile: filenames are stored as str; a non-ASCII name round-trips and keeps its insertion order on reopen"""
import zipfile
import io

_buf = io.BytesIO()
with zipfile.ZipFile(_buf, "w") as _zf:
    _zf.writestr("foo.txt", "ascii name")
    _zf.writestr("ö.txt", "unicode name")
    assert isinstance(_zf.infolist()[0].filename, str), "filename is str"

_buf.seek(0)
with zipfile.ZipFile(_buf, "r") as _zf:
    assert _zf.filelist[0].filename == "foo.txt", f"name 0 = {_zf.filelist[0].filename!r}"
    assert _zf.filelist[1].filename == "ö.txt", f"name 1 = {_zf.filelist[1].filename!r}"

print("non_ascii_filename_roundtrip OK")
"###);
    assert_output(&out, r###"non_ascii_filename_roundtrip OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/zipfile/nul_filename_truncated_at_nul.py`.
#[test]
fn test_gen_behavior_std_libs_zipfile_nul_filename_truncated_at_nul() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile"
# dimension = "behavior"
# case = "nul_filename_truncated_at_nul"
# subject = "zipfile.ZipFile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""zipfile.ZipFile: a member name containing a NUL byte is truncated at the NUL so namelist reports the prefix only"""
import zipfile
import io

_buf = io.BytesIO()
with zipfile.ZipFile(_buf, "w") as _zf:
    _zf.writestr("foo.txt\x00qqq", b"O, for a Muse of Fire!")
    assert _zf.namelist() == ["foo.txt"], f"NUL-truncated names = {_zf.namelist()!r}"

print("nul_filename_truncated_at_nul OK")
"###);
    assert_output(&out, r###"nul_filename_truncated_at_nul OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/zipfile/read_remainder_after_seek.py`.
#[test]
fn test_gen_behavior_std_libs_zipfile_read_remainder_after_seek() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile"
# dimension = "behavior"
# case = "read_remainder_after_seek"
# subject = "zipfile.ZipFile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_zipfile"
# status = "filled"
# ///
"""zipfile.ZipFile: read(-1) after a relative SEEK_CUR returns the remainder of the member from the seeked position"""
import zipfile
import io
import os

_charge = b"Charge men!"
_cloc = _charge.find(b"men")

_buf = io.BytesIO()
with zipfile.ZipFile(_buf, "w") as _zf:
    _zf.writestr("c.txt", _charge)
_buf.seek(0)
with zipfile.ZipFile(_buf, "r") as _zf:
    with _zf.open("c.txt", "r") as _fp:
        _fp.seek(_cloc, os.SEEK_CUR)
        assert _fp.read(-1) == b"men!", "read(-1) after seek"

print("read_remainder_after_seek OK")
"###);
    assert_output(&out, r###"read_remainder_after_seek OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/zipfile/stored_preserves_content.py`.
#[test]
fn test_gen_behavior_std_libs_zipfile_stored_preserves_content() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile"
# dimension = "behavior"
# case = "stored_preserves_content"
# subject = "zipfile.ZipFile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""zipfile.ZipFile: ZIP_STORED writes no compression: file_size equals compress_size and the content round-trips byte-for-byte"""
import zipfile
import io

_data = b"Hello, World! This is test data."
_buf = io.BytesIO()
with zipfile.ZipFile(_buf, "w", compression=zipfile.ZIP_STORED) as _zf:
    _zf.writestr("stored.txt", _data)

_buf.seek(0)
with zipfile.ZipFile(_buf, "r") as _zf:
    _info = _zf.getinfo("stored.txt")
    assert _info.file_size == _info.compress_size, "STORED: file_size == compress_size"
    assert _zf.read("stored.txt") == _data, "STORED: content matches"

print("stored_preserves_content OK")
"###);
    assert_output(&out, r###"stored_preserves_content OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/zipfile/testzip_returns_bad_member.py`.
#[test]
fn test_gen_behavior_std_libs_zipfile_testzip_returns_bad_member() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile"
# dimension = "behavior"
# case = "testzip_returns_bad_member"
# subject = "zipfile.ZipFile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_zipfile"
# status = "filled"
# ///
"""zipfile.ZipFile: testzip() returns the name of the first member whose stored CRC does not match its content"""
import zipfile
import io

# A crafted STORED entry "afile" whose recorded CRC is wrong.
_zip_bad_crc = (
    b"PK\x03\x04\x14\x00\x00\x00\x00\x00 \x8b\x8a;:r\xab\xff\x0c\x00\x00\x00"
    b"\x0c\x00\x00\x00\x05\x00\x00\x00afilehello,Aworld"
    b"PK\x01\x02\x14\x03\x14\x00\x00\x00\x00\x00 \x8b\x8a;:r\xab\xff\x0c\x00"
    b"\x00\x00\x0c\x00\x00\x00\x05\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00"
    b"\x80\x01\x00\x00\x00\x00afile"
    b"PK\x05\x06\x00\x00\x00\x00\x01\x00\x01\x003\x00\x00\x00/\x00\x00\x00\x00\x00"
)

with zipfile.ZipFile(io.BytesIO(_zip_bad_crc), "r") as _zf:
    assert _zf.testzip() == "afile", f"testzip = {_zf.testzip()!r}"

print("testzip_returns_bad_member OK")
"###);
    assert_output(&out, r###"testzip_returns_bad_member OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/zipfile/writestr_with_zipinfo_metadata.py`.
#[test]
fn test_gen_behavior_std_libs_zipfile_writestr_with_zipinfo_metadata() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile"
# dimension = "behavior"
# case = "writestr_with_zipinfo_metadata"
# subject = "zipfile.ZipFile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""zipfile.ZipFile: writestr accepts a ZipInfo argument carrying metadata (compress_type) and the named member reads back its content"""
import zipfile
import io

_buf = io.BytesIO()
_zi = zipfile.ZipInfo("meta.txt")
_zi.compress_type = zipfile.ZIP_STORED
with zipfile.ZipFile(_buf, "w") as _zf:
    _zf.writestr(_zi, b"metadata content")

_buf.seek(0)
with zipfile.ZipFile(_buf, "r") as _zf:
    assert _zf.read("meta.txt") == b"metadata content", "writestr with ZipInfo"

print("writestr_with_zipinfo_metadata OK")
"###);
    assert_output(&out, r###"writestr_with_zipinfo_metadata OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/zipfile/zipinfo_default_attributes.py`.
#[test]
fn test_gen_behavior_std_libs_zipfile_zipinfo_default_attributes() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile"
# dimension = "behavior"
# case = "zipinfo_default_attributes"
# subject = "zipfile.ZipInfo"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_zipfile"
# status = "filled"
# ///
"""zipfile.ZipInfo: a bare ZipInfo() carries the documented defaults: NoName filename, date_time (1980,1,1,0,0,0), ZIP_STORED compress_type, empty comment/extra, DEFAULT_VERSION create/extract version, and zero sizes/flags"""
import zipfile

_zi = zipfile.ZipInfo()
assert _zi.orig_filename == "NoName", f"orig_filename = {_zi.orig_filename!r}"
assert _zi.filename == "NoName", f"filename = {_zi.filename!r}"
assert _zi.date_time == (1980, 1, 1, 0, 0, 0), f"date_time = {_zi.date_time!r}"
assert _zi.compress_type == zipfile.ZIP_STORED, f"compress_type = {_zi.compress_type!r}"
assert _zi.comment == b"", f"comment = {_zi.comment!r}"
assert _zi.extra == b"", f"extra = {_zi.extra!r}"
assert _zi.create_system in (0, 3), f"create_system = {_zi.create_system!r}"
assert _zi.create_version == zipfile.DEFAULT_VERSION, f"create_version = {_zi.create_version!r}"
assert _zi.extract_version == zipfile.DEFAULT_VERSION, f"extract_version = {_zi.extract_version!r}"
assert _zi.reserved == 0, f"reserved = {_zi.reserved!r}"
assert _zi.flag_bits == 0, f"flag_bits = {_zi.flag_bits!r}"
assert _zi.volume == 0, f"volume = {_zi.volume!r}"
assert _zi.internal_attr == 0, f"internal_attr = {_zi.internal_attr!r}"
assert _zi.external_attr == 0, f"external_attr = {_zi.external_attr!r}"
assert _zi.file_size == 0, f"file_size = {_zi.file_size!r}"
assert _zi.compress_size == 0, f"compress_size = {_zi.compress_size!r}"

print("zipinfo_default_attributes OK")
"###);
    assert_output(&out, r###"zipinfo_default_attributes OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/zipfile/zipinfo_from_file_directory.py`.
#[test]
fn test_gen_behavior_std_libs_zipfile_zipinfo_from_file_directory() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile"
# dimension = "behavior"
# case = "zipinfo_from_file_directory"
# subject = "zipfile.ZipInfo"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_zipfile"
# status = "filled"
# ///
"""zipfile.ZipInfo: ZipInfo.from_file on a directory appends a trailing slash to the arcname, marks is_dir() True, uses ZIP_STORED, and has zero file_size"""
import zipfile
import os
import tempfile

with tempfile.TemporaryDirectory() as _td:
    _sub = os.path.join(_td, "adir")
    os.mkdir(_sub)
    _zi = zipfile.ZipInfo.from_file(_sub, "stuff")
    assert _zi.filename == "stuff/", f"dir arcname = {_zi.filename!r}"
    assert _zi.is_dir(), "dir is_dir() == True"
    assert _zi.compress_type == zipfile.ZIP_STORED, "dir compress_type"
    assert _zi.file_size == 0, f"dir file_size = {_zi.file_size!r}"

print("zipinfo_from_file_directory OK")
"###);
    assert_output(&out, r###"zipinfo_from_file_directory OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/zipfile/zipinfo_from_file_records_size.py`.
#[test]
fn test_gen_behavior_std_libs_zipfile_zipinfo_from_file_records_size() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile"
# dimension = "behavior"
# case = "zipinfo_from_file_records_size"
# subject = "zipfile.ZipInfo"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_zipfile"
# status = "filled"
# ///
"""zipfile.ZipInfo: ZipInfo.from_file on a real file records its byte size, applies the supplied arcname, and is_dir() is False"""
import zipfile
import os
import posixpath
import tempfile

with tempfile.TemporaryDirectory() as _td:
    _fpath = os.path.join(_td, "payload.bin")
    with open(_fpath, "wb") as _f:
        _f.write(b"0123456789")
    _zi = zipfile.ZipInfo.from_file(_fpath, "renamed")
    assert posixpath.basename(_zi.filename) == "renamed", \
        f"from_file arcname = {_zi.filename!r}"
    assert not _zi.is_dir(), "file is_dir() == False"
    assert _zi.file_size == 10, f"from_file size = {_zi.file_size!r}"

print("zipinfo_from_file_records_size OK")
"###);
    assert_output(&out, r###"zipinfo_from_file_records_size OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/zipfile/zipinfo_repr.py`.
#[test]
fn test_gen_behavior_std_libs_zipfile_zipinfo_repr() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile"
# dimension = "behavior"
# case = "zipinfo_repr"
# subject = "zipfile.ZipInfo"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_zipfile"
# status = "filled"
# ///
"""zipfile.ZipInfo: repr(ZipInfo(filename='empty')) is the stable string <ZipInfo filename='empty' file_size=0>"""
import zipfile

assert repr(zipfile.ZipInfo(filename="empty")) == "<ZipInfo filename='empty' file_size=0>", \
    "empty ZipInfo repr"

print("zipinfo_repr OK")
"###);
    assert_output(&out, r###"zipinfo_repr OK
"###);
}
