use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/gzip/badgzipfile_is_oserror_subclass.py`.
#[test]
fn test_gen_behavior_std_libs_gzip_badgzipfile_is_oserror_subclass() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gzip"
# dimension = "behavior"
# case = "badgzipfile_is_oserror_subclass"
# subject = "gzip.BadGzipFile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_gzip.py"
# status = "filled"
# ///
"""gzip.BadGzipFile: gzip.BadGzipFile is a subclass of OSError (the exception hierarchy contract on Python 3.12+)"""
import gzip

assert issubclass(gzip.BadGzipFile, OSError), "BadGzipFile is an OSError subclass"

print("badgzipfile_is_oserror_subclass OK")
"###);
    assert_output(&out, r###"badgzipfile_is_oserror_subclass OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/gzip/compress_decompress_roundtrips_full_byte_range.py`.
#[test]
fn test_gen_behavior_std_libs_gzip_compress_decompress_roundtrips_full_byte_range() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gzip"
# dimension = "behavior"
# case = "compress_decompress_roundtrips_full_byte_range"
# subject = "gzip.decompress"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""gzip.decompress: compress then decompress round-trips an arbitrary payload covering the full 0..255 byte range repeated 10 times, byte-for-byte"""
import gzip

_payload = bytes(range(256)) * 10
_compressed = gzip.compress(_payload)
assert gzip.decompress(_compressed) == _payload, "full byte range round-trip"

print("compress_decompress_roundtrips_full_byte_range OK")
"###);
    assert_output(&out, r###"compress_decompress_roundtrips_full_byte_range OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/gzip/compress_has_gzip_magic_and_deflate_method.py`.
#[test]
fn test_gen_behavior_std_libs_gzip_compress_has_gzip_magic_and_deflate_method() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gzip"
# dimension = "behavior"
# case = "compress_has_gzip_magic_and_deflate_method"
# subject = "gzip.compress"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""gzip.compress: a compressed stream starts with the gzip magic bytes 0x1f 0x8b and a CM byte of 0x08 (DEFLATE), and is itself a bytes object"""
import gzip

_data = b"hello world this is a test"
_compressed = gzip.compress(_data)
assert isinstance(_compressed, bytes), f"compress type = {type(_compressed)!r}"
# gzip magic bytes: 1f 8b
assert _compressed[:2] == b"\x1f\x8b", f"gzip magic = {_compressed[:2]!r}"
# CM byte (compression method) = 8 means DEFLATE.
assert _compressed[2] == 0x08, f"CM byte should be DEFLATE=8, got {_compressed[2]:#x}"

print("compress_has_gzip_magic_and_deflate_method OK")
"###);
    assert_output(&out, r###"compress_has_gzip_magic_and_deflate_method OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/gzip/compress_mtime_zero_is_deterministic.py`.
#[test]
fn test_gen_behavior_std_libs_gzip_compress_mtime_zero_is_deterministic() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gzip"
# dimension = "behavior"
# case = "compress_mtime_zero_is_deterministic"
# subject = "gzip.compress"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""gzip.compress: two compress calls on the same data with mtime=0 produce byte-identical streams (no embedded wall-clock timestamp)"""
import gzip

_d3 = b"deterministic test"
_a = gzip.compress(_d3, mtime=0)
_b = gzip.compress(_d3, mtime=0)
assert _a == _b, "mtime=0 deterministic"

print("compress_mtime_zero_is_deterministic OK")
"###);
    assert_output(&out, r###"compress_mtime_zero_is_deterministic OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/gzip/compress_shrinks_compressible_data.py`.
#[test]
fn test_gen_behavior_std_libs_gzip_compress_shrinks_compressible_data() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gzip"
# dimension = "behavior"
# case = "compress_shrinks_compressible_data"
# subject = "gzip.compress"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""gzip.compress: compressing highly repetitive data at level 6 yields a stream strictly smaller than the uncompressed input"""
import gzip

_compressible = b"hello " * 500
_comp = gzip.compress(_compressible, compresslevel=6)
assert len(_comp) < len(_compressible), "compressed < uncompressed"

print("compress_shrinks_compressible_data OK")
"###);
    assert_output(&out, r###"compress_shrinks_compressible_data OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/gzip/empty_bytes_roundtrip.py`.
#[test]
fn test_gen_behavior_std_libs_gzip_empty_bytes_roundtrip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gzip"
# dimension = "behavior"
# case = "empty_bytes_roundtrip"
# subject = "gzip.decompress"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""gzip.decompress: compressing then decompressing empty bytes round-trips to empty bytes"""
import gzip

_empty = gzip.compress(b"")
assert gzip.decompress(_empty) == b"", "empty round-trip"

print("empty_bytes_roundtrip OK")
"###);
    assert_output(&out, r###"empty_bytes_roundtrip OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/gzip/gzipfile_bytesio_roundtrip.py`.
#[test]
fn test_gen_behavior_std_libs_gzip_gzipfile_bytesio_roundtrip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gzip"
# dimension = "behavior"
# case = "gzipfile_bytesio_roundtrip"
# subject = "gzip.GzipFile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""gzip.GzipFile: GzipFile over a BytesIO fileobj writes in 'wb' and reads back the same bytes in 'rb'"""
import gzip
import io

_buf = io.BytesIO()
with gzip.GzipFile(fileobj=_buf, mode="wb") as _w:
    _w.write(b"buffer data")
_buf.seek(0)
with gzip.GzipFile(fileobj=_buf, mode="rb") as _r:
    _out = _r.read()
assert _out == b"buffer data", f"BytesIO GzipFile = {_out!r}"

print("gzipfile_bytesio_roundtrip OK")
"###);
    assert_output(&out, r###"gzipfile_bytesio_roundtrip OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/gzip/gzipfile_capability_flags_and_name.py`.
#[test]
fn test_gen_behavior_std_libs_gzip_gzipfile_capability_flags_and_name() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gzip"
# dimension = "behavior"
# case = "gzipfile_capability_flags_and_name"
# subject = "gzip.GzipFile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""gzip.GzipFile: a GzipFile over a nameless fileobj reports name=='' , mode==WRITE/READ, the complementary readable/writable/seekable flags, closed transitions across the with-block, and fileno() raises UnsupportedOperation with no underlying fd"""
import gzip
import io

# A GzipFile wrapping a nameless fileobj reports name == "".
_buf = io.BytesIO()
with gzip.GzipFile(fileobj=_buf, mode="wb") as _w:
    _w.write(b"payload" * 20)
    assert _w.name == "", f"write name = {_w.name!r}"
    assert _w.mode == gzip.WRITE, f"write mode = {_w.mode!r}"
    assert _w.readable() is False, "writer not readable"
    assert _w.writable() is True, "writer writable"
    assert _w.seekable() is True, "writer seekable"
    assert _w.closed is False, "open writer not closed"
    # No underlying fd: fileno() raises while open.
    try:
        _w.fileno()
        raise AssertionError("expected UnsupportedOperation")
    except io.UnsupportedOperation:
        pass

# After the context exits the object reports closed.
assert _w.closed is True, "writer closed after with-block"

# Reading reports the complementary capability flags.
_buf.seek(0)
with gzip.GzipFile(fileobj=_buf, mode="rb") as _r:
    assert _r.read() == b"payload" * 20, "read round-trip"
    assert _r.mode == gzip.READ, f"read mode = {_r.mode!r}"
    assert _r.readable() is True, "reader readable"
    assert _r.writable() is False, "reader not writable"
    assert _r.seekable() is True, "reader seekable"

print("gzipfile_capability_flags_and_name OK")
"###);
    assert_output(&out, r###"gzipfile_capability_flags_and_name OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/gzip/gzipfile_decodes_fextra_header.py`.
#[test]
fn test_gen_behavior_std_libs_gzip_gzipfile_decodes_fextra_header() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gzip"
# dimension = "behavior"
# case = "gzipfile_decodes_fextra_header"
# subject = "gzip.GzipFile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_gzip.py"
# status = "filled"
# ///
"""gzip.GzipFile: a gzip member carrying an optional FEXTRA field (header flag 0x04) is decoded by skipping the extra field and still recovering the payload"""
import gzip
import io

# A gzip member may carry an optional FEXTRA field (flag bit 0x04 in the
# header). The decoder must skip it and still recover the payload. This
# fixed blob encodes b"Test" with a 5-byte extra field.
_with_extra = (
    b"\x1f\x8b\x08\x04\xb2\x17cQ\x02\xff\x05\x00Extra"
    b"\x0bI-.\x01\x002\xd1Mx\x04\x00\x00\x00"
)
with gzip.GzipFile(fileobj=io.BytesIO(_with_extra)) as _f:
    assert _f.read() == b"Test", "FEXTRA header decode"

print("gzipfile_decodes_fextra_header OK")
"###);
    assert_output(&out, r###"gzipfile_decodes_fextra_header OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/gzip/gzipfile_flush_prefix_incomplete_until_close.py`.
#[test]
fn test_gen_behavior_std_libs_gzip_gzipfile_flush_prefix_incomplete_until_close() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gzip"
# dimension = "behavior"
# case = "gzipfile_flush_prefix_incomplete_until_close"
# subject = "gzip.GzipFile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""gzip.GzipFile: flush() commits a syncable prefix but the trailer is only written on close: the closed stream decompresses while the flushed-only prefix raises EOFError"""
import gzip
import io

# flush() on a writer commits a syncable prefix, but the trailer is only
# written on close. The fully closed stream decompresses; the flushed-
# but-not-closed prefix is incomplete and raises EOFError.
_sink = io.BytesIO()
_msg = b"flushed content"
with gzip.GzipFile(fileobj=_sink, mode="w") as _w:
    _w.write(_msg)
    _w.flush()
    _partial = _sink.getvalue()
_full = _sink.getvalue()

assert gzip.decompress(_full) == _msg, "closed stream decompresses"
try:
    gzip.decompress(_partial)
    raise AssertionError("expected EOFError on flushed-only prefix")
except EOFError:
    pass

print("gzipfile_flush_prefix_incomplete_until_close OK")
"###);
    assert_output(&out, r###"gzipfile_flush_prefix_incomplete_until_close OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/gzip/gzipfile_multi_write_single_stream.py`.
#[test]
fn test_gen_behavior_std_libs_gzip_gzipfile_multi_write_single_stream() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gzip"
# dimension = "behavior"
# case = "gzipfile_multi_write_single_stream"
# subject = "gzip.GzipFile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""gzip.GzipFile: two successive writes to one GzipFile concatenate into a single decompressed stream"""
import gzip
import io

_buf = io.BytesIO()
with gzip.GzipFile(fileobj=_buf, mode="wb") as _w:
    _w.write(b"part1 ")
    _w.write(b"part2")
_buf.seek(0)
with gzip.GzipFile(fileobj=_buf, mode="rb") as _r:
    _out = _r.read()
assert _out == b"part1 part2", f"multi-write = {_out!r}"

print("gzipfile_multi_write_single_stream OK")
"###);
    assert_output(&out, r###"gzipfile_multi_write_single_stream OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/gzip/gzipfile_read_truncated_stream_raises_eoferror.py`.
#[test]
fn test_gen_behavior_std_libs_gzip_gzipfile_read_truncated_stream_raises_eoferror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gzip"
# dimension = "behavior"
# case = "gzipfile_read_truncated_stream_raises_eoferror"
# subject = "gzip.GzipFile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_gzip.py"
# status = "filled"
# ///
"""gzip.GzipFile: reading a stream missing its 8-byte trailer through GzipFile raises EOFError once the reader walks past the payload, while a sized read of exactly the payload length still recovers the bytes; header-only prefixes raise EOFError on first read"""
import gzip
import io

_data = b"line of repeated content\n" * 50
# Drop the 8-byte trailer (CRC32 + ISIZE) to truncate the stream.
_truncated = gzip.compress(_data)[:-8]

# Reading the whole stream walks past the missing trailer -> EOFError.
with gzip.GzipFile(fileobj=io.BytesIO(_truncated)) as _f:
    try:
        _f.read()
        raise AssertionError("expected EOFError on full read")
    except EOFError:
        pass

# Reading exactly the payload length succeeds; the next read hits the
# missing trailer and raises EOFError.
with gzip.GzipFile(fileobj=io.BytesIO(_truncated)) as _f:
    assert _f.read(len(_data)) == _data, "payload bytes recovered"
    try:
        _f.read(1)
        raise AssertionError("expected EOFError past payload")
    except EOFError:
        pass

# Streams truncated inside the header (only a few bytes) also raise
# EOFError on the first read.
for _n in range(2, 10):
    with gzip.GzipFile(fileobj=io.BytesIO(_truncated[:_n])) as _f:
        try:
            _f.read(1)
            raise AssertionError(f"expected EOFError at prefix {_n}")
        except EOFError:
            pass

print("gzipfile_read_truncated_stream_raises_eoferror OK")
"###);
    assert_output(&out, r###"gzipfile_read_truncated_stream_raises_eoferror OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/gzip/gzipfile_write_returns_count_and_tell_tracks.py`.
#[test]
fn test_gen_behavior_std_libs_gzip_gzipfile_write_returns_count_and_tell_tracks() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gzip"
# dimension = "behavior"
# case = "gzipfile_write_returns_count_and_tell_tracks"
# subject = "gzip.GzipFile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""gzip.GzipFile: GzipFile.write() accepts any buffer (array.array) and returns the byte count while tell() tracks the uncompressed position"""
import array
import gzip
import io

# write() accepts any buffer (e.g. array.array) and returns the byte
# count; tell() tracks the uncompressed position.
_q = array.array("Q", [1, 2, 3, 4, 5])
_nbytes = len(_q) * _q.itemsize
with gzip.GzipFile(fileobj=io.BytesIO(), mode="w") as _wb:
    assert _wb.write(_q) == _nbytes, f"write returned {_wb.write!r}"
    assert _wb.tell() == _nbytes, f"tell = {_wb.tell()!r}"

print("gzipfile_write_returns_count_and_tell_tracks OK")
"###);
    assert_output(&out, r###"gzipfile_write_returns_count_and_tell_tracks OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/gzip/gzipfile_write_seek_write_doubles.py`.
#[test]
fn test_gen_behavior_std_libs_gzip_gzipfile_write_seek_write_doubles() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gzip"
# dimension = "behavior"
# case = "gzipfile_write_seek_write_doubles"
# subject = "gzip.GzipFile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""gzip.GzipFile: write, seek forward by the data length, then write again: the closed stream decompresses to the message repeated twice"""
import gzip
import io

# write, seek forward by the data length, write again: the stream
# decompresses to the message repeated twice.
_msg = b"important message here."
_sink = io.BytesIO()
with gzip.GzipFile(fileobj=_sink, mode="w") as _sw:
    _sw.write(_msg)
    _sw.seek(len(_msg))
    _sw.write(_msg)
assert gzip.decompress(_sink.getvalue()) == _msg * 2, "write/seek/write"

print("gzipfile_write_seek_write_doubles OK")
"###);
    assert_output(&out, r###"gzipfile_write_seek_write_doubles OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/gzip/higher_level_smaller_or_equal_same_output.py`.
#[test]
fn test_gen_behavior_std_libs_gzip_higher_level_smaller_or_equal_same_output() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gzip"
# dimension = "behavior"
# case = "higher_level_smaller_or_equal_same_output"
# subject = "gzip.compress"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""gzip.compress: compresslevel 1 and 9 both decompress back to the same input, and for repetitive data level 9 produces output no larger than level 1"""
import gzip

_data = b"aaaa" * 1000
_c1 = gzip.compress(_data, compresslevel=1)
_c9 = gzip.compress(_data, compresslevel=9)
assert gzip.decompress(_c1) == _data, "level 1 correct"
assert gzip.decompress(_c9) == _data, "level 9 correct"
# Higher compression should produce smaller or equal output for repetitive data.
assert len(_c9) <= len(_c1), f"level 9 <= level 1: {len(_c9)} vs {len(_c1)}"

print("higher_level_smaller_or_equal_same_output OK")
"###);
    assert_output(&out, r###"higher_level_smaller_or_equal_same_output OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/gzip/open_fileobj_read_modes.py`.
#[test]
fn test_gen_behavior_std_libs_gzip_open_fileobj_read_modes() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gzip"
# dimension = "behavior"
# case = "open_fileobj_read_modes"
# subject = "gzip.open"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""gzip.open: gzip.open accepts an in-memory BytesIO fileobj and honours 'r'/'rb' (bytes) and 'rt' (decoded str) read modes over the same compressed payload"""
import gzip
import io

_raw = b"the quick brown fox\n" * 25
_text = _raw.decode("ascii")
_compressed = gzip.compress(_raw)

with gzip.open(io.BytesIO(_compressed), "r") as _f:
    assert _f.read() == _raw, "open fileobj mode r"
with gzip.open(io.BytesIO(_compressed), "rb") as _f:
    assert _f.read() == _raw, "open fileobj mode rb"
with gzip.open(io.BytesIO(_compressed), "rt", encoding="ascii") as _f:
    assert _f.read() == _text, "open fileobj mode rt"

print("open_fileobj_read_modes OK")
"###);
    assert_output(&out, r###"open_fileobj_read_modes OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/gzip/open_path_binary_roundtrip.py`.
#[test]
fn test_gen_behavior_std_libs_gzip_open_path_binary_roundtrip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gzip"
# dimension = "behavior"
# case = "open_path_binary_roundtrip"
# subject = "gzip.open"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""gzip.open: gzip.open on a tempfile path writes bytes in 'wb' and reads them back identically in 'rb'"""
import gzip
import os
import tempfile

with tempfile.TemporaryDirectory() as _d:
    _p = os.path.join(_d, "data.gz")
    with gzip.open(_p, "wb") as _gf:
        _gf.write(b"gzipped content")
    with gzip.open(_p, "rb") as _gf2:
        _content = _gf2.read()
    assert _content == b"gzipped content", f"gzip.open round-trip = {_content!r}"

print("open_path_binary_roundtrip OK")
"###);
    assert_output(&out, r###"open_path_binary_roundtrip OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/gzip/open_path_text_roundtrip.py`.
#[test]
fn test_gen_behavior_std_libs_gzip_open_path_text_roundtrip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gzip"
# dimension = "behavior"
# case = "open_path_text_roundtrip"
# subject = "gzip.open"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""gzip.open: gzip.open on a tempfile path in 'wt' writes str and 'rt' reads the same str back"""
import gzip
import os
import tempfile

with tempfile.TemporaryDirectory() as _d:
    _p = os.path.join(_d, "data.gz")
    with gzip.open(_p, "wt") as _gf:
        _gf.write("text content\n")
    with gzip.open(_p, "rt") as _gf2:
        _content = _gf2.read()
    assert _content == "text content\n", f"text mode = {_content!r}"

print("open_path_text_roundtrip OK")
"###);
    assert_output(&out, r###"open_path_text_roundtrip OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/gzip/test_gzip__test_gzip_bad_gzip_file_exception.py`.
#[test]
fn test_gen_behavior_std_libs_gzip_test_gzip__test_gzip_bad_gzip_file_exception() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gzip"
# dimension = "behavior"
# case = "test_gzip__test_gzip_bad_gzip_file_exception"
# subject = "cpython.test_gzip.TestGzip.test_gzip_BadGzipFile_exception"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_gzip.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_gzip.py::TestGzip::test_gzip_BadGzipFile_exception
"""Auto-ported test: TestGzip::test_gzip_BadGzipFile_exception (CPython 3.12 oracle)."""


import array
import functools
import gc
import io
import os
import struct
import sys
import unittest
from subprocess import PIPE, Popen
from test.support import catch_unraisable_exception
from test.support import import_helper
from test.support import os_helper
from test.support import _4G, bigmemtest, requires_subprocess
from test.support.script_helper import assert_python_ok, assert_python_failure


'Test script for the gzip module.\n'

gzip = import_helper.import_module('gzip')

zlib = import_helper.import_module('zlib')

data1 = b'  int length=DEFAULTALLOC, err = Z_OK;\n  PyObject *RetVal;\n  int flushmode = Z_FINISH;\n  unsigned long start_total_out;\n\n'

data2 = b'/* zlibmodule.c -- gzip-compatible data compression */\n/* See http://www.gzip.org/zlib/\n/* See http://www.winimage.com/zLibDll for Windows */\n'

TEMPDIR = os.path.abspath(os_helper.TESTFN) + '-gzdir'

class UnseekableIO(io.BytesIO):

    def seekable(self):
        return False

    def tell(self):
        raise io.UnsupportedOperation

    def seek(self, *args):
        raise io.UnsupportedOperation

class BaseTest(unittest.TestCase):
    filename = os_helper.TESTFN

    def setUp(self):
        os_helper.unlink(self.filename)

    def tearDown(self):
        os_helper.unlink(self.filename)

def create_and_remove_directory(directory):

    def decorator(function):

        @functools.wraps(function)
        def wrapper(*args, **kwargs):
            os.makedirs(directory)
            try:
                return function(*args, **kwargs)
            finally:
                os_helper.rmtree(directory)
        return wrapper
    return decorator


# --- test body ---
filename = os_helper.TESTFN
os_helper.unlink(filename)

assert issubclass(gzip.BadGzipFile, OSError)
print("TestGzip::test_gzip_BadGzipFile_exception: ok")
"###);
    assert_output(&out, r###"TestGzip::test_gzip_BadGzipFile_exception: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/gzip/test_gzip__test_refloop_unraisable.py`.
#[test]
fn test_gen_behavior_std_libs_gzip_test_gzip__test_refloop_unraisable() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gzip"
# dimension = "behavior"
# case = "test_gzip__test_refloop_unraisable"
# subject = "cpython.test_gzip.TestGzip.test_refloop_unraisable"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_gzip.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_gzip.py::TestGzip::test_refloop_unraisable
"""Auto-ported test: TestGzip::test_refloop_unraisable (CPython 3.12 oracle)."""


import array
import functools
import gc
import io
import os
import struct
import sys
import unittest
from subprocess import PIPE, Popen
from test.support import catch_unraisable_exception
from test.support import import_helper
from test.support import os_helper
from test.support import _4G, bigmemtest, requires_subprocess
from test.support.script_helper import assert_python_ok, assert_python_failure


'Test script for the gzip module.\n'

gzip = import_helper.import_module('gzip')

zlib = import_helper.import_module('zlib')

data1 = b'  int length=DEFAULTALLOC, err = Z_OK;\n  PyObject *RetVal;\n  int flushmode = Z_FINISH;\n  unsigned long start_total_out;\n\n'

data2 = b'/* zlibmodule.c -- gzip-compatible data compression */\n/* See http://www.gzip.org/zlib/\n/* See http://www.winimage.com/zLibDll for Windows */\n'

TEMPDIR = os.path.abspath(os_helper.TESTFN) + '-gzdir'

class UnseekableIO(io.BytesIO):

    def seekable(self):
        return False

    def tell(self):
        raise io.UnsupportedOperation

    def seek(self, *args):
        raise io.UnsupportedOperation

class BaseTest(unittest.TestCase):
    filename = os_helper.TESTFN

    def setUp(self):
        os_helper.unlink(self.filename)

    def tearDown(self):
        os_helper.unlink(self.filename)

def create_and_remove_directory(directory):

    def decorator(function):

        @functools.wraps(function)
        def wrapper(*args, **kwargs):
            os.makedirs(directory)
            try:
                return function(*args, **kwargs)
            finally:
                os_helper.rmtree(directory)
        return wrapper
    return decorator


# --- test body ---
filename = os_helper.TESTFN
os_helper.unlink(filename)
with catch_unraisable_exception() as cm:
    gzip.GzipFile(fileobj=io.BytesIO(), mode='w')
    gc.collect()

    assert cm.unraisable is None
print("TestGzip::test_refloop_unraisable: ok")
"###);
    assert_output(&out, r###"TestGzip::test_refloop_unraisable: ok
"###);
}
