use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/tarfile/addfile_extractfile_roundtrip.py`.
#[test]
fn test_gen_behavior_std_libs_tarfile_addfile_extractfile_roundtrip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tarfile"
# dimension = "behavior"
# case = "addfile_extractfile_roundtrip"
# subject = "tarfile.TarFile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tarfile.py"
# status = "filled"
# ///
"""tarfile.TarFile: content added via addfile is retrieved byte-for-byte via extractfile after a write -> seek(0) -> read round-trip"""
import tarfile
import io

_buf = io.BytesIO()
_data = b"Round-trip test data for tarfile."
with tarfile.open(fileobj=_buf, mode="w") as _tf:
    _ti = tarfile.TarInfo("roundtrip.txt")
    _ti.size = len(_data)
    _tf.addfile(_ti, io.BytesIO(_data))
_buf.seek(0)
with tarfile.open(fileobj=_buf, mode="r") as _tf:
    _fh = _tf.extractfile("roundtrip.txt")
    assert _fh is not None, "extractfile not None"
    assert _fh.read() == _data, "round-trip data"

print("addfile_extractfile_roundtrip OK")
"###);
    assert_output(&out, r###"addfile_extractfile_roundtrip OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/tarfile/context_manager_tracks_closed.py`.
#[test]
fn test_gen_behavior_std_libs_tarfile_context_manager_tracks_closed() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tarfile"
# dimension = "behavior"
# case = "context_manager_tracks_closed"
# subject = "tarfile.TarFile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tarfile.py"
# status = "filled"
# ///
"""tarfile.TarFile: an open TarFile reports .closed False, .close() flips it to True, and re-entering an already-closed TarFile as a context manager raises OSError"""
import tarfile
import io

_buf = io.BytesIO()
with tarfile.open(fileobj=_buf, mode="w") as _tf:
    _ti = tarfile.TarInfo("a.txt")
    _ti.size = 1
    _tf.addfile(_ti, io.BytesIO(b"x"))
_buf.seek(0)

_tf = tarfile.open(fileobj=_buf, mode="r")
assert not _tf.closed, "open archive not closed"
_tf.close()
assert _tf.closed, "closed archive is closed"

_raised = False
try:
    with _tf:
        pass
except OSError:
    _raised = True
assert _raised, "reusing closed TarFile raises OSError"

print("context_manager_tracks_closed OK")
"###);
    assert_output(&out, r###"context_manager_tracks_closed OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/tarfile/create_pax_header_magic_prefix.py`.
#[test]
fn test_gen_behavior_std_libs_tarfile_create_pax_header_magic_prefix() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tarfile"
# dimension = "behavior"
# case = "create_pax_header_magic_prefix"
# subject = "tarfile.TarInfo"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tarfile.py"
# status = "filled"
# ///
"""tarfile.TarInfo: create_pax_header emits a bytes block beginning with the PAX extended-header magic name './@PaxHeader'"""
import tarfile

_ti = tarfile.TarInfo("foo")
_ti.mtime = 1000.1
_ti.size = 100
_ti.uid = 123
_ti.gid = 124
_info = _ti.get_info()

_hdr = _ti.create_pax_header(_info, encoding="iso8859-1")
assert isinstance(_hdr, (bytes, bytearray)), f"header type = {type(_hdr)!r}"
assert _hdr.startswith(b"././@PaxHeader"), f"header prefix = {_hdr[:14]!r}"

print("create_pax_header_magic_prefix OK")
"###);
    assert_output(&out, r###"create_pax_header_magic_prefix OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/tarfile/data_filter_strips_leading_slash.py`.
#[test]
fn test_gen_behavior_std_libs_tarfile_data_filter_strips_leading_slash() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tarfile"
# dimension = "behavior"
# case = "data_filter_strips_leading_slash"
# subject = "tarfile.data_filter"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tarfile.py"
# status = "filled"
# ///
"""tarfile.data_filter: data_filter strips a leading absolute slash ('/etc/evil.txt' -> 'etc/evil.txt') rather than letting the path escape to root"""
import tarfile

_abs = tarfile.TarInfo("/etc/evil.txt")
_abs.size = 0
_stripped = tarfile.data_filter(_abs, "dest")
assert _stripped.name == "etc/evil.txt", f"abs strip = {_stripped.name!r}"

print("data_filter_strips_leading_slash OK")
"###);
    assert_output(&out, r###"data_filter_strips_leading_slash OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/tarfile/extractall_writes_tree_to_disk.py`.
#[test]
fn test_gen_behavior_std_libs_tarfile_extractall_writes_tree_to_disk() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tarfile"
# dimension = "behavior"
# case = "extractall_writes_tree_to_disk"
# subject = "tarfile.TarFile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tarfile.py"
# status = "filled"
# ///
"""tarfile.TarFile: extractall(filter='data') materializes both a top-level file and a nested subdir/file onto a TemporaryDirectory with the original content"""
import tarfile
import io
import os
import tempfile

with tempfile.TemporaryDirectory() as _tmpdir:
    _buf = io.BytesIO()
    with tarfile.open(fileobj=_buf, mode="w") as _tf:
        for _fname, _fc in [("file1.txt", b"one"), ("subdir/file2.txt", b"two")]:
            _ti = tarfile.TarInfo(_fname)
            _ti.size = len(_fc)
            _tf.addfile(_ti, io.BytesIO(_fc))
    _buf.seek(0)
    with tarfile.open(fileobj=_buf, mode="r") as _tf:
        _tf.extractall(_tmpdir, filter="data")
    assert os.path.exists(os.path.join(_tmpdir, "file1.txt")), "file1.txt extracted"
    assert os.path.exists(os.path.join(_tmpdir, "subdir", "file2.txt")), "subdir/file2.txt"
    with open(os.path.join(_tmpdir, "file1.txt"), "rb") as _f:
        assert _f.read() == b"one", "extracted file1 content"

print("extractall_writes_tree_to_disk OK")
"###);
    assert_output(&out, r###"extractall_writes_tree_to_disk OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/tarfile/fully_trusted_filter_is_identity.py`.
#[test]
fn test_gen_behavior_std_libs_tarfile_fully_trusted_filter_is_identity() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tarfile"
# dimension = "behavior"
# case = "fully_trusted_filter_is_identity"
# subject = "tarfile.fully_trusted_filter"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tarfile.py"
# status = "filled"
# ///
"""tarfile.fully_trusted_filter: fully_trusted_filter returns the same TarInfo object it was given (identity), applying no sanitization"""
import tarfile
import io

# Build a small in-memory archive: a file and a directory.
_buf = io.BytesIO()
with tarfile.open(fileobj=_buf, mode="w") as _tf:
    _f = tarfile.TarInfo("safe.txt")
    _f.size = 3
    _tf.addfile(_f, io.BytesIO(b"abc"))
    _d = tarfile.TarInfo("dir/")
    _d.type = tarfile.DIRTYPE
    _tf.addfile(_d)

_buf.seek(0)
with tarfile.open(fileobj=_buf, mode="r") as _tf:
    for _m in _tf.getmembers():
        _out = tarfile.fully_trusted_filter(_m, "")
        assert _out is _m, f"fully_trusted not identity for {_m.name!r}"

print("fully_trusted_filter_is_identity OK")
"###);
    assert_output(&out, r###"fully_trusted_filter_is_identity OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/tarfile/get_info_exposes_metadata.py`.
#[test]
fn test_gen_behavior_std_libs_tarfile_get_info_exposes_metadata() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tarfile"
# dimension = "behavior"
# case = "get_info_exposes_metadata"
# subject = "tarfile.TarInfo"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tarfile.py"
# status = "filled"
# ///
"""tarfile.TarInfo: TarInfo.get_info() returns the header-building metadata dict carrying name/size/uid/gid and a fractional mtime preserved verbatim"""
import tarfile

_ti = tarfile.TarInfo("foo")
_ti.mtime = 1000.1
_ti.size = 100
_ti.uid = 123
_ti.gid = 124
_info = _ti.get_info()
assert _info["name"] == "foo", f"info name = {_info['name']!r}"
assert _info["size"] == 100, f"info size = {_info['size']!r}"
assert _info["uid"] == 123, f"info uid = {_info['uid']!r}"
assert _info["gid"] == 124, f"info gid = {_info['gid']!r}"
# mtime keeps its fractional part in the info dict.
assert _info["mtime"] == 1000.1, f"info mtime = {_info['mtime']!r}"

print("get_info_exposes_metadata OK")
"###);
    assert_output(&out, r###"get_info_exposes_metadata OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/tarfile/getnames_getmembers_shapes.py`.
#[test]
fn test_gen_behavior_std_libs_tarfile_getnames_getmembers_shapes() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tarfile"
# dimension = "behavior"
# case = "getnames_getmembers_shapes"
# subject = "tarfile.TarFile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tarfile.py"
# status = "filled"
# ///
"""tarfile.TarFile: getnames returns a list of name strings (including a subdir path) and getmembers returns a matching list of TarInfo objects; getmember/extractfile resolve a member by name and TarInfo carries name/size/mode/mtime/type"""
import tarfile
import io

_buf = io.BytesIO()
with tarfile.open(fileobj=_buf, mode="w") as _tf:
    _c1 = b"Hello, tar world!"
    _i1 = tarfile.TarInfo(name="hello.txt")
    _i1.size = len(_c1)
    _tf.addfile(_i1, io.BytesIO(_c1))
    _c2 = b"Second file content here."
    _i2 = tarfile.TarInfo(name="subdir/second.txt")
    _i2.size = len(_c2)
    _tf.addfile(_i2, io.BytesIO(_c2))

_buf.seek(0)
with tarfile.open(fileobj=_buf, mode="r") as _tf:
    _names = _tf.getnames()
    assert isinstance(_names, list), f"getnames type = {type(_names)!r}"
    assert "hello.txt" in _names, f"hello.txt in names = {_names!r}"
    assert "subdir/second.txt" in _names, "subdir/second.txt in names"

    _members = _tf.getmembers()
    assert isinstance(_members, list), f"getmembers type = {type(_members)!r}"
    assert len(_members) == 2, f"two members = {len(_members)!r}"

    _ti = _tf.getmember("hello.txt")
    assert isinstance(_ti, tarfile.TarInfo), f"getmember type = {type(_ti)!r}"
    assert hasattr(_ti, "name"), "TarInfo has name"
    assert hasattr(_ti, "size"), "TarInfo has size"
    assert hasattr(_ti, "mode"), "TarInfo has mode"
    assert hasattr(_ti, "mtime"), "TarInfo has mtime"
    assert hasattr(_ti, "type"), "TarInfo has type"
    assert _ti.name == "hello.txt", f"name = {_ti.name!r}"
    assert _ti.size == 17, f"size = {_ti.size!r}"

    _fh = _tf.extractfile("hello.txt")
    assert _fh is not None, "extractfile not None"
    assert _fh.read() == b"Hello, tar world!", "extractfile data"

print("getnames_getmembers_shapes OK")
"###);
    assert_output(&out, r###"getnames_getmembers_shapes OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/tarfile/gnu_lifts_name_limits.py`.
#[test]
fn test_gen_behavior_std_libs_tarfile_gnu_lifts_name_limits() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tarfile"
# dimension = "behavior"
# case = "gnu_lifts_name_limits"
# subject = "tarfile.TarInfo"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tarfile.py"
# status = "filled"
# ///
"""tarfile.TarInfo: GNU_FORMAT encodes long names and linknames that overflow USTAR, so tobuf(GNU_FORMAT) succeeds where USTAR would raise"""
import tarfile

# GNU encodes long names, so an overlong name now succeeds.
tarfile.TarInfo("123/" * 126 + "longname").tobuf(tarfile.GNU_FORMAT)

# GNU also encodes long linknames.
_ti = tarfile.TarInfo("longlink")
_ti.linkname = "123/" * 126 + "longname"
_ti.tobuf(tarfile.GNU_FORMAT)

print("gnu_lifts_name_limits OK")
"###);
    assert_output(&out, r###"gnu_lifts_name_limits OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/tarfile/gzip_compresses_repeated_bytes.py`.
#[test]
fn test_gen_behavior_std_libs_tarfile_gzip_compresses_repeated_bytes() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tarfile"
# dimension = "behavior"
# case = "gzip_compresses_repeated_bytes"
# subject = "tarfile.TarFile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tarfile.py"
# status = "filled"
# ///
"""tarfile.TarFile: a w:gz archive of 10000 identical bytes is smaller on the wire than the same payload written to a plain (uncompressed) w archive"""
import tarfile
import io

_data = b"x" * 10000  # compressible data

_buf_gz = io.BytesIO()
with tarfile.open(fileobj=_buf_gz, mode="w:gz") as _tf:
    _ti = tarfile.TarInfo("compressed.txt")
    _ti.size = len(_data)
    _tf.addfile(_ti, io.BytesIO(_data))
_size_gz = _buf_gz.tell()

_buf_plain = io.BytesIO()
with tarfile.open(fileobj=_buf_plain, mode="w") as _tf:
    _tip = tarfile.TarInfo("plain.txt")
    _tip.size = len(_data)
    _tf.addfile(_tip, io.BytesIO(_data))
_size_plain = _buf_plain.tell()

assert _size_gz < _size_plain, "gzip compresses repeated bytes"

print("gzip_compresses_repeated_bytes OK")
"###);
    assert_output(&out, r###"gzip_compresses_repeated_bytes OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/tarfile/is_tarfile_preserves_stream_position.py`.
#[test]
fn test_gen_behavior_std_libs_tarfile_is_tarfile_preserves_stream_position() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tarfile"
# dimension = "behavior"
# case = "is_tarfile_preserves_stream_position"
# subject = "tarfile.is_tarfile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tarfile.py"
# status = "filled"
# ///
"""tarfile.is_tarfile: is_tarfile returns True for a valid in-memory tar stream and leaves the file-like object's seek position at 0 (untouched)"""
import tarfile
import io

_buf = io.BytesIO()
with tarfile.open(fileobj=_buf, mode="w") as _tf:
    _ti = tarfile.TarInfo("b.txt")
    _ti.size = 1
    _tf.addfile(_ti, io.BytesIO(b"y"))

_flo = io.BytesIO(_buf.getvalue())
_flo.seek(0)
assert tarfile.is_tarfile(_flo), "is_tarfile valid stream"
assert _flo.tell() == 0, f"position preserved = {_flo.tell()!r}"

print("is_tarfile_preserves_stream_position OK")
"###);
    assert_output(&out, r###"is_tarfile_preserves_stream_position OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/tarfile/itn_encodes_octal_and_base256.py`.
#[test]
fn test_gen_behavior_std_libs_tarfile_itn_encodes_octal_and_base256() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tarfile"
# dimension = "behavior"
# case = "itn_encodes_octal_and_base256"
# subject = "tarfile.itn"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tarfile.py"
# status = "filled"
# ///
"""tarfile.itn: itn writes the octal form by default (1, 2**21-1) and the GNU base-256 form for values that overflow octal or are negative (2**21, -1, -100)"""
import tarfile

# Default form is octal.
assert tarfile.itn(1) == b"0000001\x00", "itn 1"
assert tarfile.itn(2097151) == b"7777777\x00", "itn octal max"

# Values that overflow octal need GNU base-256 (returns a bytearray, which
# compares equal to bytes).
assert tarfile.itn(2097152, format=tarfile.GNU_FORMAT) == b"\x80\x00\x00\x00\x00 \x00\x00", "itn gnu 2**21"
assert tarfile.itn(-1, format=tarfile.GNU_FORMAT) == b"\xff\xff\xff\xff\xff\xff\xff\xff", "itn gnu -1"
assert tarfile.itn(-100, format=tarfile.GNU_FORMAT) == b"\xff\xff\xff\xff\xff\xff\xff\x9c", "itn gnu -100"

print("itn_encodes_octal_and_base256 OK")
"###);
    assert_output(&out, r###"itn_encodes_octal_and_base256 OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/tarfile/itn_nti_roundtrip_gnu_range.py`.
#[test]
fn test_gen_behavior_std_libs_tarfile_itn_nti_roundtrip_gnu_range() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tarfile"
# dimension = "behavior"
# case = "itn_nti_roundtrip_gnu_range"
# subject = "tarfile.itn"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tarfile.py"
# status = "filled"
# ///
"""tarfile.itn: itn and nti are inverses across the GNU range: nti(itn(n, GNU_FORMAT)) == n for representative n including 0, 2**21-1, 2**21, 2**32-1, -1, -100"""
import tarfile

for n in (0, 1, 2097151, 2097152, 4294967295, -1, -100):
    enc = tarfile.itn(n, format=tarfile.GNU_FORMAT)
    assert tarfile.nti(enc) == n, f"round-trip {n} -> {enc!r}"

print("itn_nti_roundtrip_gnu_range OK")
"###);
    assert_output(&out, r###"itn_nti_roundtrip_gnu_range OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/tarfile/multiple_members_roundtrip.py`.
#[test]
fn test_gen_behavior_std_libs_tarfile_multiple_members_roundtrip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tarfile"
# dimension = "behavior"
# case = "multiple_members_roundtrip"
# subject = "tarfile.TarFile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tarfile.py"
# status = "filled"
# ///
"""tarfile.TarFile: three distinct members written into one archive are each retrieved with their own content intact via extractfile"""
import tarfile
import io

_files = {"a.txt": b"alpha", "b.txt": b"beta", "c.txt": b"gamma"}

_buf = io.BytesIO()
with tarfile.open(fileobj=_buf, mode="w") as _tf:
    for _name, _content in _files.items():
        _ti = tarfile.TarInfo(_name)
        _ti.size = len(_content)
        _tf.addfile(_ti, io.BytesIO(_content))
_buf.seek(0)
with tarfile.open(fileobj=_buf, mode="r") as _tf:
    for _name, _content in _files.items():
        _fh = _tf.extractfile(_name)
        assert _fh is not None, f"extractfile {_name}"
        assert _fh.read() == _content, f"content mismatch: {_name}"

print("multiple_members_roundtrip OK")
"###);
    assert_output(&out, r###"multiple_members_roundtrip OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/tarfile/next_on_empty_archive_is_none.py`.
#[test]
fn test_gen_behavior_std_libs_tarfile_next_on_empty_archive_is_none() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tarfile"
# dimension = "behavior"
# case = "next_on_empty_archive_is_none"
# subject = "tarfile.TarFile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tarfile.py"
# status = "filled"
# ///
"""tarfile.TarFile: next() returns None at the end of an empty archive in both seekable ('r') and streaming ('r|') read modes"""
import tarfile
import io

_buf = io.BytesIO()
tarfile.open(fileobj=_buf, mode="w").close()

_buf.seek(0)
with tarfile.open(fileobj=_buf, mode="r|") as _tf:
    assert _tf.next() is None, "stream next on empty"

_buf.seek(0)
with tarfile.open(fileobj=_buf, mode="r") as _tf:
    assert _tf.next() is None, "seek next on empty"

print("next_on_empty_archive_is_none OK")
"###);
    assert_output(&out, r###"next_on_empty_archive_is_none OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/tarfile/nti_decodes_octal_and_base256.py`.
#[test]
fn test_gen_behavior_std_libs_tarfile_nti_decodes_octal_and_base256() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tarfile"
# dimension = "behavior"
# case = "nti_decodes_octal_and_base256"
# subject = "tarfile.nti"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tarfile.py"
# status = "filled"
# ///
"""tarfile.nti: nti reads octal-ASCII number fields (1, 2**21-1, 0 for nul/spaces) and GNU base-256 fields with the high bit set (2**21, -1, -100)"""
import tarfile

# Octal-ASCII form.
assert tarfile.nti(b"0000001\x00") == 1, "octal 1"
assert tarfile.nti(b"7777777\x00") == 2097151, "octal max (2**21-1)"
assert tarfile.nti(b"\x00") == 0, "single nul -> 0"
assert tarfile.nti(b"       \x00") == 0, "all spaces -> 0"

# GNU base-256 form (high bit of byte 0 set) reaches values octal cannot,
# including negatives.
assert tarfile.nti(b"\x80\x00\x00\x00\x00 \x00\x00") == 2097152, "base-256 2**21"
assert tarfile.nti(b"\xff\xff\xff\xff\xff\xff\xff\xff") == -1, "base-256 -1"
assert tarfile.nti(b"\xff\xff\xff\xff\xff\xff\xff\x9c") == -100, "base-256 -100"

print("nti_decodes_octal_and_base256 OK")
"###);
    assert_output(&out, r###"nti_decodes_octal_and_base256 OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/tarfile/pax_has_no_practical_field_limits.py`.
#[test]
fn test_gen_behavior_std_libs_tarfile_pax_has_no_practical_field_limits() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tarfile"
# dimension = "behavior"
# case = "pax_has_no_practical_field_limits"
# subject = "tarfile.TarInfo"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tarfile.py"
# status = "filled"
# ///
"""tarfile.TarInfo: PAX_FORMAT carries oversized names and a 2**56 uid in extended headers, so tobuf(PAX_FORMAT) raises nothing where USTAR/GNU would"""
import tarfile

# PAX extended headers carry oversized values; nothing raises.
tarfile.TarInfo("123/" * 126 + "longname").tobuf(tarfile.PAX_FORMAT)

_ti = tarfile.TarInfo("name")
_ti.uid = 72057594037927936  # 2**56, overflows GNU base-256 octal/uid field
_ti.tobuf(tarfile.PAX_FORMAT)

print("pax_has_no_practical_field_limits OK")
"###);
    assert_output(&out, r###"pax_has_no_practical_field_limits OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/tarfile/pax_roundtrips_custom_headers.py`.
#[test]
fn test_gen_behavior_std_libs_tarfile_pax_roundtrips_custom_headers() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tarfile"
# dimension = "behavior"
# case = "pax_roundtrips_custom_headers"
# subject = "tarfile.TarFile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tarfile.py"
# status = "filled"
# ///
"""tarfile.TarFile: a custom pax_headers key/value pair round-trips by key through a PAX-format archive"""
import tarfile
import io

_buf = io.BytesIO()
with tarfile.open(fileobj=_buf, mode="w", format=tarfile.PAX_FORMAT) as _tf:
    _x = tarfile.TarInfo("x")
    _x.size = 0
    _x.pax_headers["VENDOR.note"] = "hi"
    _tf.addfile(_x)
_buf.seek(0)
with tarfile.open(fileobj=_buf, mode="r") as _tf:
    _m = _tf.getmember("x")
    assert _m.pax_headers.get("VENDOR.note") == "hi", f"pax meta = {_m.pax_headers!r}"

print("pax_roundtrips_custom_headers OK")
"###);
    assert_output(&out, r###"pax_roundtrips_custom_headers OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/tarfile/pax_roundtrips_nonascii_name.py`.
#[test]
fn test_gen_behavior_std_libs_tarfile_pax_roundtrips_nonascii_name() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tarfile"
# dimension = "behavior"
# case = "pax_roundtrips_nonascii_name"
# subject = "tarfile.TarFile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tarfile.py"
# status = "filled"
# ///
"""tarfile.TarFile: a non-ASCII member name survives a PAX-format write/read round-trip via the PAX path= record"""
import tarfile
import io

_name = "foo㍴"  # contains a non-ASCII CJK compatibility character

_buf = io.BytesIO()
with tarfile.open(fileobj=_buf, mode="w", format=tarfile.PAX_FORMAT) as _tf:
    _u = tarfile.TarInfo(_name)
    _u.size = 0
    _tf.addfile(_u)
_buf.seek(0)
with tarfile.open(fileobj=_buf, mode="r") as _tf:
    assert _tf.getnames() == [_name], f"unicode name = {_tf.getnames()!r}"

print("pax_roundtrips_nonascii_name OK")
"###);
    assert_output(&out, r###"pax_roundtrips_nonascii_name OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/tarfile/pax_roundtrips_oversized_numeric_fields.py`.
#[test]
fn test_gen_behavior_std_libs_tarfile_pax_roundtrips_oversized_numeric_fields() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tarfile"
# dimension = "behavior"
# case = "pax_roundtrips_oversized_numeric_fields"
# subject = "tarfile.TarFile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tarfile.py"
# status = "filled"
# ///
"""tarfile.TarFile: a PAX-format archive round-trips uid and mtime values too large for the classic ustar octal fields (uid=2**24, mtime=10**10), recovering the exact integers and payload"""
import tarfile
import io

_buf = io.BytesIO()
with tarfile.open(fileobj=_buf, mode="w", format=tarfile.PAX_FORMAT) as _tf:
    _big = tarfile.TarInfo("big.bin")
    _big.size = 5
    _big.uid = 16777216  # 8**8, overflows the 7-digit octal uid field
    _big.mtime = 10**10
    _tf.addfile(_big, io.BytesIO(b"hello"))
_buf.seek(0)
with tarfile.open(fileobj=_buf, mode="r") as _tf:
    _m = _tf.getmember("big.bin")
    assert _m.uid == 16777216, f"pax uid = {_m.uid!r}"
    assert _m.mtime == 10**10, f"pax mtime = {_m.mtime!r}"
    assert _tf.extractfile("big.bin").read() == b"hello", "pax data"

print("pax_roundtrips_oversized_numeric_fields OK")
"###);
    assert_output(&out, r###"pax_roundtrips_oversized_numeric_fields OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/tarfile/stn_nts_pad_and_truncate.py`.
#[test]
fn test_gen_behavior_std_libs_tarfile_stn_nts_pad_and_truncate() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tarfile"
# dimension = "behavior"
# case = "stn_nts_pad_and_truncate"
# subject = "tarfile.stn"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tarfile.py"
# status = "filled"
# ///
"""tarfile.stn: stn nul-pads a string into a fixed-width field and truncates an overlong one; nts decodes a nul-terminated field back, stopping at the first nul"""
import tarfile

# stn: encode a string into a fixed-width nul-padded field (truncating).
assert tarfile.stn("foo", 8, "ascii", "strict") == b"foo\x00\x00\x00\x00\x00", "stn pad"
assert tarfile.stn("foobar", 3, "ascii", "strict") == b"foo", "stn truncate"

# nts: decode a nul-terminated field back to a string (stops at first nul).
assert tarfile.nts(b"foo\x00\x00\x00\x00\x00", "ascii", "strict") == "foo", "nts pad"
assert tarfile.nts(b"foo\x00bar\x00", "ascii", "strict") == "foo", "nts stops at nul"

print("stn_nts_pad_and_truncate OK")
"###);
    assert_output(&out, r###"stn_nts_pad_and_truncate OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/tarfile/tar_and_data_filters_preserve_name_type.py`.
#[test]
fn test_gen_behavior_std_libs_tarfile_tar_and_data_filters_preserve_name_type() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tarfile"
# dimension = "behavior"
# case = "tar_and_data_filters_preserve_name_type"
# subject = "tarfile.data_filter"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tarfile.py"
# status = "filled"
# ///
"""tarfile.data_filter: tar_filter and data_filter return sanitized copies that preserve a safe member's name and type"""
import tarfile
import io

# Build a small in-memory archive: a file and a directory.
_buf = io.BytesIO()
with tarfile.open(fileobj=_buf, mode="w") as _tf:
    _f = tarfile.TarInfo("safe.txt")
    _f.size = 3
    _tf.addfile(_f, io.BytesIO(b"abc"))
    _d = tarfile.TarInfo("dir/")
    _d.type = tarfile.DIRTYPE
    _tf.addfile(_d)

_buf.seek(0)
with tarfile.open(fileobj=_buf, mode="r") as _tf:
    for _m in _tf.getmembers():
        _t = tarfile.tar_filter(_m, "")
        assert _t.name == _m.name, f"tar_filter name {_t.name!r}"
        assert _t.type == _m.type, f"tar_filter type {_t.type!r}"
        _data = tarfile.data_filter(_m, "")
        assert _data.name == _m.name, f"data_filter name {_data.name!r}"
        assert _data.type == _m.type, f"data_filter type {_data.type!r}"

print("tar_and_data_filters_preserve_name_type OK")
"###);
    assert_output(&out, r###"tar_and_data_filters_preserve_name_type OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/tarfile/type_distinguishes_file_from_dir.py`.
#[test]
fn test_gen_behavior_std_libs_tarfile_type_distinguishes_file_from_dir() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tarfile"
# dimension = "behavior"
# case = "type_distinguishes_file_from_dir"
# subject = "tarfile.TarFile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tarfile.py"
# status = "filled"
# ///
"""tarfile.TarFile: a member tagged REGTYPE reads back as isreg() while a member tagged DIRTYPE reads back as isdir(), and TarInfo.type round-trips each constant"""
import tarfile
import io

_buf = io.BytesIO()
with tarfile.open(fileobj=_buf, mode="w") as _tf:
    _ti_file = tarfile.TarInfo("regular.txt")
    _ti_file.type = tarfile.REGTYPE
    _ti_file.size = 0
    _tf.addfile(_ti_file)
    _ti_dir = tarfile.TarInfo("mydir/")
    _ti_dir.type = tarfile.DIRTYPE
    _tf.addfile(_ti_dir)
_buf.seek(0)
with tarfile.open(fileobj=_buf, mode="r") as _tf:
    _reg = _tf.getmember("regular.txt")
    _dir = _tf.getmember("mydir/")
    assert _reg.type == tarfile.REGTYPE, f"regular type = {_reg.type!r}"
    assert _dir.type == tarfile.DIRTYPE, f"dir type = {_dir.type!r}"
    assert _reg.isreg(), "isreg for regular"
    assert _dir.isdir(), "isdir for directory"

print("type_distinguishes_file_from_dir OK")
"###);
    assert_output(&out, r###"type_distinguishes_file_from_dir OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/tarfile/ustar_name_within_limits_ok.py`.
#[test]
fn test_gen_behavior_std_libs_tarfile_ustar_name_within_limits_ok() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tarfile"
# dimension = "behavior"
# case = "ustar_name_within_limits_ok"
# subject = "tarfile.TarInfo"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tarfile.py"
# status = "filled"
# ///
"""tarfile.TarInfo: USTAR accepts a 100-char name and a name that splits into prefix(<=155)+name(<=100); tobuf(USTAR_FORMAT) succeeds for both"""
import tarfile

# A 100-char name fits within the USTAR name field.
tarfile.TarInfo("0123456789" * 10).tobuf(tarfile.USTAR_FORMAT)

# A name splittable into prefix(<=155) + name(<=100) is fine.
tarfile.TarInfo("123/" * 62 + "longname").tobuf(tarfile.USTAR_FORMAT)

print("ustar_name_within_limits_ok OK")
"###);
    assert_output(&out, r###"ustar_name_within_limits_ok OK
"###);
}
