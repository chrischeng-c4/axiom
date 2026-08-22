use super::super::super::super::harness::*;

/// Ported from `tests/cpython/errors/std-libs/zipfile/bad_compression_raises_notimplementederror.py`.
#[test]
fn test_gen_errors_std_libs_zipfile_bad_compression_raises_notimplementederror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile"
# dimension = "errors"
# case = "bad_compression_raises_notimplementederror"
# subject = "zipfile.ZipFile"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""zipfile.ZipFile: bad_compression_raises_notimplementederror (errors)."""
import zipfile
import io

_raised = False
try:
    zipfile.ZipFile(io.BytesIO(), 'w', -1)
except NotImplementedError:
    _raised = True
assert _raised, "bad_compression_raises_notimplementederror: expected NotImplementedError"
print("bad_compression_raises_notimplementederror OK")
"###);
    assert_output(&out, r###"bad_compression_raises_notimplementederror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/zipfile/bad_constructor_mode_raises_valueerror.py`.
#[test]
fn test_gen_errors_std_libs_zipfile_bad_constructor_mode_raises_valueerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile"
# dimension = "errors"
# case = "bad_constructor_mode_raises_valueerror"
# subject = "zipfile.ZipFile"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""zipfile.ZipFile: bad_constructor_mode_raises_valueerror (errors)."""
import zipfile
import io

_raised = False
try:
    zipfile.ZipFile(io.BytesIO(), 'q')
except ValueError:
    _raised = True
assert _raised, "bad_constructor_mode_raises_valueerror: expected ValueError"
print("bad_constructor_mode_raises_valueerror OK")
"###);
    assert_output(&out, r###"bad_constructor_mode_raises_valueerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/zipfile/bad_crc_read_raises_badzipfile.py`.
#[test]
fn test_gen_errors_std_libs_zipfile_bad_crc_read_raises_badzipfile() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile"
# dimension = "errors"
# case = "bad_crc_read_raises_badzipfile"
# subject = "zipfile.ZipFile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_zipfile"
# status = "filled"
# ///
"""zipfile.ZipFile: a crafted STORED entry whose recorded CRC does not match its content raises BadZipFile on read(), and also on a chunked open()/read() stream"""
import zipfile
import io

# A STORED entry "afile" whose recorded CRC does not match its content.
_zip_bad_crc = (
    b"PK\x03\x04\x14\x00\x00\x00\x00\x00 \x8b\x8a;:r\xab\xff\x0c\x00\x00\x00"
    b"\x0c\x00\x00\x00\x05\x00\x00\x00afilehello,Aworld"
    b"PK\x01\x02\x14\x03\x14\x00\x00\x00\x00\x00 \x8b\x8a;:r\xab\xff\x0c\x00"
    b"\x00\x00\x0c\x00\x00\x00\x05\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00"
    b"\x80\x01\x00\x00\x00\x00afile"
    b"PK\x05\x06\x00\x00\x00\x00\x01\x00\x01\x003\x00\x00\x00/\x00\x00\x00\x00\x00"
)

# read() of a bad-CRC member raises BadZipFile.
with zipfile.ZipFile(io.BytesIO(_zip_bad_crc), "r") as _zf:
    _raised = False
    try:
        _zf.read("afile")
    except zipfile.BadZipFile:
        _raised = True
    assert _raised, "read of bad CRC -> BadZipFile"

# Streaming the member chunk by chunk also raises BadZipFile.
with zipfile.ZipFile(io.BytesIO(_zip_bad_crc), "r") as _zf:
    _raised = False
    try:
        with _zf.open("afile", "r") as _fp:
            while _fp.read(2):
                pass
    except zipfile.BadZipFile:
        _raised = True
    assert _raised, "streamed read of bad CRC -> BadZipFile"

print("bad_crc_read_raises_badzipfile OK")
"###);
    assert_output(&out, r###"bad_crc_read_raises_badzipfile OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/zipfile/bad_open_modes_raise_valueerror.py`.
#[test]
fn test_gen_errors_std_libs_zipfile_bad_open_modes_raise_valueerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile"
# dimension = "errors"
# case = "bad_open_modes_raise_valueerror"
# subject = "zipfile.ZipFile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""zipfile.ZipFile: ZipFile.open() rejects the bogus per-member modes 'q', 'U', 'rU' with ValueError"""
import zipfile
import io

_buf = io.BytesIO()
with zipfile.ZipFile(_buf, "w") as _z:
    _z.writestr("foo.txt", "data")
_buf.seek(0)
with zipfile.ZipFile(_buf, "r") as _z:
    for _mode in ("q", "U", "rU"):
        _raised = False
        try:
            _z.open("foo.txt", _mode)
        except ValueError:
            _raised = True
        assert _raised, f"open mode {_mode!r} -> ValueError"

print("bad_open_modes_raise_valueerror OK")
"###);
    assert_output(&out, r###"bad_open_modes_raise_valueerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/zipfile/empty_file_raises_badzipfile.py`.
#[test]
fn test_gen_errors_std_libs_zipfile_empty_file_raises_badzipfile() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile"
# dimension = "errors"
# case = "empty_file_raises_badzipfile"
# subject = "zipfile.ZipFile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""zipfile.ZipFile: opening a zero-byte file in read mode raises BadZipFile"""
import zipfile
import os
import tempfile

with tempfile.TemporaryDirectory() as _td:
    _empty = os.path.join(_td, "empty.zip")
    open(_empty, "wb").close()
    _raised = False
    try:
        zipfile.ZipFile(_empty)
    except zipfile.BadZipFile:
        _raised = True
    assert _raised, "empty file -> BadZipFile"

print("empty_file_raises_badzipfile OK")
"###);
    assert_output(&out, r###"empty_file_raises_badzipfile OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/zipfile/invalid_data_raises_badzipfile.py`.
#[test]
fn test_gen_errors_std_libs_zipfile_invalid_data_raises_badzipfile() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile"
# dimension = "errors"
# case = "invalid_data_raises_badzipfile"
# subject = "zipfile.ZipFile"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""zipfile.ZipFile: invalid_data_raises_badzipfile (errors)."""
import zipfile
import io

_raised = False
try:
    zipfile.ZipFile(io.BytesIO(b'not a zip'))
except zipfile.BadZipFile:
    _raised = True
assert _raised, "invalid_data_raises_badzipfile: expected zipfile.BadZipFile"
print("invalid_data_raises_badzipfile OK")
"###);
    assert_output(&out, r###"invalid_data_raises_badzipfile OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/zipfile/missing_path_raises_oserror.py`.
#[test]
fn test_gen_errors_std_libs_zipfile_missing_path_raises_oserror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile"
# dimension = "errors"
# case = "missing_path_raises_oserror"
# subject = "zipfile.ZipFile"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""zipfile.ZipFile: missing_path_raises_oserror (errors)."""
import zipfile
import os
import tempfile

_raised = False
try:
    zipfile.ZipFile(os.path.join(tempfile.mkdtemp(), 'nope.zip'))
except OSError:
    _raised = True
assert _raised, "missing_path_raises_oserror: expected OSError"
print("missing_path_raises_oserror OK")
"###);
    assert_output(&out, r###"missing_path_raises_oserror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/zipfile/operations_after_close_raise_valueerror.py`.
#[test]
fn test_gen_errors_std_libs_zipfile_operations_after_close_raise_valueerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile"
# dimension = "errors"
# case = "operations_after_close_raise_valueerror"
# subject = "zipfile.ZipFile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""zipfile.ZipFile: after close(), each of read/open/testzip/writestr on the ZipFile raises ValueError"""
import zipfile
import io


def _expect_valueerror(fn):
    try:
        fn()
    except ValueError:
        return True
    return False


_buf = io.BytesIO()
_z = zipfile.ZipFile(_buf, "w")
_z.writestr("foo.txt", "data")
_z.close()

assert _expect_valueerror(lambda: _z.read("foo.txt")), "read after close -> ValueError"
assert _expect_valueerror(lambda: _z.open("foo.txt")), "open after close -> ValueError"
assert _expect_valueerror(lambda: _z.testzip()), "testzip after close -> ValueError"
assert _expect_valueerror(lambda: _z.writestr("b.txt", "x")), "writestr after close -> ValueError"

print("operations_after_close_raise_valueerror OK")
"###);
    assert_output(&out, r###"operations_after_close_raise_valueerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/zipfile/pre_1980_date_raises_valueerror.py`.
#[test]
fn test_gen_errors_std_libs_zipfile_pre_1980_date_raises_valueerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile"
# dimension = "errors"
# case = "pre_1980_date_raises_valueerror"
# subject = "zipfile.ZipInfo"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""zipfile.ZipInfo: pre_1980_date_raises_valueerror (errors)."""
import zipfile

_raised = False
try:
    zipfile.ZipInfo('old', (1979, 1, 1, 0, 0, 0))
except ValueError:
    _raised = True
assert _raised, "pre_1980_date_raises_valueerror: expected ValueError"
print("pre_1980_date_raises_valueerror OK")
"###);
    assert_output(&out, r###"pre_1980_date_raises_valueerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/zipfile/read_missing_member_raises_keyerror.py`.
#[test]
fn test_gen_errors_std_libs_zipfile_read_missing_member_raises_keyerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile"
# dimension = "errors"
# case = "read_missing_member_raises_keyerror"
# subject = "zipfile.ZipFile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""zipfile.ZipFile: reading a member that is not in the archive raises KeyError; open() of the same missing member also raises KeyError"""
import zipfile
import io

_buf = io.BytesIO()
with zipfile.ZipFile(_buf, "w") as _z:
    _z.writestr("a.txt", "hello")
_buf.seek(0)
with zipfile.ZipFile(_buf, "r") as _z:
    _read_raised = False
    try:
        _z.read("missing.txt")
    except KeyError:
        _read_raised = True
    assert _read_raised, "missing member read -> KeyError"

    _open_raised = False
    try:
        _z.open("missing.txt", "r")
    except KeyError:
        _open_raised = True
    assert _open_raised, "missing member open -> KeyError"

print("read_missing_member_raises_keyerror OK")
"###);
    assert_output(&out, r###"read_missing_member_raises_keyerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/zipfile/str_comment_raises_typeerror.py`.
#[test]
fn test_gen_errors_std_libs_zipfile_str_comment_raises_typeerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile"
# dimension = "errors"
# case = "str_comment_raises_typeerror"
# subject = "zipfile.ZipFile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""zipfile.ZipFile: assigning a str (not bytes) to ZipFile.comment raises TypeError"""
import zipfile
import io

_buf = io.BytesIO()
with zipfile.ZipFile(_buf, "w") as _z:
    _raised = False
    try:
        _z.comment = "not bytes"
    except TypeError:
        _raised = True
    assert _raised, "str comment -> TypeError"

print("str_comment_raises_typeerror OK")
"###);
    assert_output(&out, r###"str_comment_raises_typeerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/zipfile/truncated_archive_raises_badzipfile.py`.
#[test]
fn test_gen_errors_std_libs_zipfile_truncated_archive_raises_badzipfile() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile"
# dimension = "errors"
# case = "truncated_archive_raises_badzipfile"
# subject = "zipfile.ZipFile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_zipfile"
# status = "filled"
# ///
"""zipfile.ZipFile: every proper prefix of a valid archive (truncated to n bytes for all n) fails to open with BadZipFile"""
import zipfile
import io

_good = io.BytesIO()
with zipfile.ZipFile(_good, "w") as _zf:
    _zf.writestr("foo.txt", b"O, for a Muse of Fire!")
_blob = _good.getvalue()

for _n in range(len(_blob)):
    _raised = False
    try:
        zipfile.ZipFile(io.BytesIO(_blob[:_n]))
    except zipfile.BadZipFile:
        _raised = True
    assert _raised, f"truncated to {_n} bytes -> BadZipFile"

print("truncated_archive_raises_badzipfile OK")
"###);
    assert_output(&out, r###"truncated_archive_raises_badzipfile OK
"###);
}
