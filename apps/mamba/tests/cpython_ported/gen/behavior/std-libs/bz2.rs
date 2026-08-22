use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/bz2/bz2file_append_multistream.py`.
#[test]
fn test_gen_behavior_std_libs_bz2_bz2file_append_multistream() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bz2"
# dimension = "behavior"
# case = "bz2file_append_multistream"
# subject = "bz2.BZ2File"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bz2.py"
# status = "filled"
# ///
"""bz2.BZ2File: appending (ab mode) concatenates bzip2 streams and a reader sees the two payloads joined in order"""
import bz2
import io

buf = io.BytesIO()
with bz2.BZ2File(buf, "wb") as f:
    f.write(b"foo")
with bz2.BZ2File(buf, "ab") as f:
    f.write(b"bar")
buf.seek(0)
with bz2.BZ2File(buf, "rb") as f:
    assert f.read() == b"foobar", "append/multi-stream ordering"
print("bz2file_append_multistream OK")
"###);
    assert_output(&out, r###"bz2file_append_multistream OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/bz2/bz2file_capability_flags.py`.
#[test]
fn test_gen_behavior_std_libs_bz2_bz2file_capability_flags() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bz2"
# dimension = "behavior"
# case = "bz2file_capability_flags"
# subject = "bz2.BZ2File"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bz2.py"
# status = "filled"
# ///
"""bz2.BZ2File: readable/writable flags reflect the open mode and write methods reject a read-only BZ2File"""
import bz2
import io

buf = io.BytesIO()
with bz2.BZ2File(buf, "wb") as f:
    assert (f.readable(), f.writable()) == (False, True), "write-mode flags"
assert f.closed is True, "closed after context exit"
buf.seek(0)
with bz2.BZ2File(buf, "rb") as f:
    assert (f.readable(), f.writable()) == (True, False), "read-mode flags"
    for op in (lambda: f.write(b"x"), lambda: f.writelines([b"x"])):
        try:
            op()
            raise AssertionError("expected OSError writing read-only file")
        except OSError:
            pass
print("bz2file_capability_flags OK")
"###);
    assert_output(&out, r###"bz2file_capability_flags OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/bz2/bz2file_closed_raises_valueerror.py`.
#[test]
fn test_gen_behavior_std_libs_bz2_bz2file_closed_raises_valueerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bz2"
# dimension = "behavior"
# case = "bz2file_closed_raises_valueerror"
# subject = "bz2.BZ2File"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bz2.py"
# status = "filled"
# ///
"""bz2.BZ2File: operating on a closed BZ2File raises ValueError and closed is True after the context exits"""
import bz2
import io

buf = io.BytesIO()
with bz2.BZ2File(buf, "wb") as f:
    f.write(b"0123456789abcdef")
assert f.closed is True, "closed after context exit"
buf.seek(0)
closed = bz2.BZ2File(buf, "rb")
closed.close()
try:
    closed.read()
    raise AssertionError("expected ValueError on closed file")
except ValueError:
    pass
print("bz2file_closed_raises_valueerror OK")
"###);
    assert_output(&out, r###"bz2file_closed_raises_valueerror OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/bz2/bz2file_mixed_iter_read.py`.
#[test]
fn test_gen_behavior_std_libs_bz2_bz2file_mixed_iter_read() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bz2"
# dimension = "behavior"
# case = "bz2file_mixed_iter_read"
# subject = "bz2.BZ2File"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bz2.py"
# status = "filled"
# ///
"""bz2.BZ2File: BZ2File iteration (next) and read share one position so readline, next, and read consume in order"""
import bz2
import io

src = io.BytesIO()
with bz2.BZ2File(src, "wb") as f:
    f.write(b"alpha\nbeta\ngamma\n")
src.seek(0)
with bz2.BZ2File(src, "rb") as f:
    f.readline()
    assert next(f) == b"beta\n", "next after readline"
    assert f.read() == b"gamma\n", "read tail"
print("bz2file_mixed_iter_read OK")
"###);
    assert_output(&out, r###"bz2file_mixed_iter_read OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/bz2/bz2file_peek_readinto.py`.
#[test]
fn test_gen_behavior_std_libs_bz2_bz2file_peek_readinto() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bz2"
# dimension = "behavior"
# case = "bz2file_peek_readinto"
# subject = "bz2.BZ2File"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bz2.py"
# status = "filled"
# ///
"""bz2.BZ2File: BZ2File peek returns a leading prefix and readinto fills a bytearray and advances the position"""
import bz2
import io

buf = io.BytesIO()
with bz2.BZ2File(buf, "wb") as f:
    f.write(b"0123456789abcdef")
buf.seek(0)
with bz2.BZ2File(buf, "rb") as f:
    peek = f.peek()
    assert len(peek) != 0 and b"0123456789abcdef".startswith(peek), "peek prefix"
    ba = bytearray(8)
    assert f.readinto(ba) == 8, "readinto count"
    assert bytes(ba) == b"01234567", f"readinto bytes = {bytes(ba)!r}"
    assert f.read() == b"89abcdef", "read remainder"
print("bz2file_peek_readinto OK")
"###);
    assert_output(&out, r###"bz2file_peek_readinto OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/bz2/bz2file_readline_readlines.py`.
#[test]
fn test_gen_behavior_std_libs_bz2_bz2file_readline_readlines() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bz2"
# dimension = "behavior"
# case = "bz2file_readline_readlines"
# subject = "bz2.BZ2File"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bz2.py"
# status = "filled"
# ///
"""bz2.BZ2File: BZ2File supports readline then readlines, splitting a multi-line payload correctly"""
import bz2
import io

buf = io.BytesIO()
with bz2.BZ2File(buf, "wb") as f:
    f.write(b"alpha\nbeta\ngamma\n")
buf.seek(0)
with bz2.BZ2File(buf, "rb") as f:
    line = f.readline()
    assert line == b"alpha\n", f"readline = {line!r}"
    rest = f.readlines()
    assert rest == [b"beta\n", b"gamma\n"], f"readlines = {rest!r}"
print("bz2file_readline_readlines OK")
"###);
    assert_output(&out, r###"bz2file_readline_readlines OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/bz2/bz2file_seek_modes.py`.
#[test]
fn test_gen_behavior_std_libs_bz2_bz2file_seek_modes() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bz2"
# dimension = "behavior"
# case = "bz2file_seek_modes"
# subject = "bz2.BZ2File"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bz2.py"
# status = "filled"
# ///
"""bz2.BZ2File: BZ2File seek works absolute, relative (whence=1) and from-end (whence=2) on a seekable read stream"""
import bz2
import io

buf = io.BytesIO()
with bz2.BZ2File(buf, "wb") as f:
    f.write(b"0123456789abcdef")
buf.seek(0)
with bz2.BZ2File(buf, "rb") as f:
    assert f.seekable() is True, "seekable in read mode"
    f.seek(8)
    assert f.read(4) == b"89ab", "seek forward absolute"
    # Now at position 12; rewind 8 bytes relative to land back at 4.
    f.seek(-8, 1)
    assert f.read(4) == b"4567", "seek backward relative"
    f.seek(-4, 2)
    assert f.read() == b"cdef", "seek from end"
print("bz2file_seek_modes OK")
"###);
    assert_output(&out, r###"bz2file_seek_modes OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/bz2/bz2file_wrap_bytesio_stays_open.py`.
#[test]
fn test_gen_behavior_std_libs_bz2_bz2file_wrap_bytesio_stays_open() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bz2"
# dimension = "behavior"
# case = "bz2file_wrap_bytesio_stays_open"
# subject = "bz2.BZ2File"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bz2.py"
# status = "filled"
# ///
"""bz2.BZ2File: closing a BZ2File that wraps a BytesIO leaves the underlying object open and the bytes valid"""
import bz2
import io

outer = io.BytesIO()
with bz2.BZ2File(outer, "w") as f:
    f.write(b"payload")
assert outer.closed is False, "underlying BytesIO stays open"
assert bz2.decompress(outer.getvalue()) == b"payload", "wrapped write round-trip"
print("bz2file_wrap_bytesio_stays_open OK")
"###);
    assert_output(&out, r###"bz2file_wrap_bytesio_stays_open OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/bz2/bz2file_write_returns_length.py`.
#[test]
fn test_gen_behavior_std_libs_bz2_bz2file_write_returns_length() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bz2"
# dimension = "behavior"
# case = "bz2file_write_returns_length"
# subject = "bz2.BZ2File"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bz2.py"
# status = "filled"
# ///
"""bz2.BZ2File: BZ2File.write returns the byte length (incl. buffer-protocol objects) and tell reflects bytes written"""
import array
import bz2
import io

q = array.array("Q", [1, 2, 3, 4, 5])
length = len(q) * q.itemsize
with bz2.BZ2File(io.BytesIO(), "w") as f:
    assert f.write(q) == length, "write returns buffer byte length"
    assert f.tell() == length, f"tell = {f.tell()}"
print("bz2file_write_returns_length OK")
"###);
    assert_output(&out, r###"bz2file_write_returns_length OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/bz2/compress_decompress_roundtrip.py`.
#[test]
fn test_gen_behavior_std_libs_bz2_compress_decompress_roundtrip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bz2"
# dimension = "behavior"
# case = "compress_decompress_roundtrip"
# subject = "bz2.compress"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bz2.py"
# status = "filled"
# ///
"""bz2.compress: compress then decompress preserves data exactly and shrinks repetitive input"""
import bz2

data = b"The quick brown fox jumps over the lazy dog\n" * 50
compressed = bz2.compress(data)
assert bz2.decompress(compressed) == data, "compress/decompress round-trip"
assert len(compressed) < len(data), "compression reduces size for repetitive data"
print("compress_decompress_roundtrip OK")
"###);
    assert_output(&out, r###"compress_decompress_roundtrip OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/bz2/decompress_multistream_concat.py`.
#[test]
fn test_gen_behavior_std_libs_bz2_decompress_multistream_concat() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bz2"
# dimension = "behavior"
# case = "decompress_multistream_concat"
# subject = "bz2.decompress"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bz2.py"
# status = "filled"
# ///
"""bz2.decompress: bz2.decompress fully consumes two concatenated bzip2 streams into the joined payload"""
import bz2

two = bz2.compress(b"AAAA") + bz2.compress(b"BBBB")
assert bz2.decompress(two) == b"AAAABBBB", "multi-stream decompress"
print("decompress_multistream_concat OK")
"###);
    assert_output(&out, r###"decompress_multistream_concat OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/bz2/decompressor_eof_transitions.py`.
#[test]
fn test_gen_behavior_std_libs_bz2_decompressor_eof_transitions() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bz2"
# dimension = "behavior"
# case = "decompressor_eof_transitions"
# subject = "bz2.BZ2Decompressor"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bz2.py"
# status = "filled"
# ///
"""bz2.BZ2Decompressor: BZ2Decompressor.eof is False initially and True after a complete stream is decompressed"""
import bz2

decomp = bz2.BZ2Decompressor()
assert decomp.eof is False, "eof False initially"
small = bz2.compress(b"small")
decomp.decompress(small)
assert decomp.eof is True, "eof True after decompression"
print("decompressor_eof_transitions OK")
"###);
    assert_output(&out, r###"decompressor_eof_transitions OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/bz2/decompressor_max_length_bounded.py`.
#[test]
fn test_gen_behavior_std_libs_bz2_decompressor_max_length_bounded() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bz2"
# dimension = "behavior"
# case = "decompressor_max_length_bounded"
# subject = "bz2.BZ2Decompressor"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bz2.py"
# status = "filled"
# ///
"""bz2.BZ2Decompressor: max_length caps per-call output and needs_input/eof track progress while draining the rest"""
import bz2

blob = bz2.compress(b"x" * 1000)
decomp = bz2.BZ2Decompressor()
part = decomp.decompress(blob, max_length=10)
assert len(part) == 10, f"max_length cap = {len(part)}"
assert decomp.needs_input is False, "needs_input False while output pending"
assert decomp.eof is False, "not eof mid-stream"
rest = decomp.decompress(b"", max_length=-1)
assert part + rest == b"x" * 1000, "bounded reassembly"
assert decomp.eof is True, "eof after draining"
print("decompressor_max_length_bounded OK")
"###);
    assert_output(&out, r###"decompressor_max_length_bounded OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/bz2/decompressor_unused_data_trailing.py`.
#[test]
fn test_gen_behavior_std_libs_bz2_decompressor_unused_data_trailing() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bz2"
# dimension = "behavior"
# case = "decompressor_unused_data_trailing"
# subject = "bz2.BZ2Decompressor"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bz2.py"
# status = "filled"
# ///
"""bz2.BZ2Decompressor: trailing bytes after a complete stream are surfaced via unused_data, not decompressed"""
import bz2

data = bz2.compress(b"payload") + b"extra_trailing_bytes"
decomp = bz2.BZ2Decompressor()
result = decomp.decompress(data)
assert result == b"payload", f"payload = {result!r}"
assert decomp.unused_data == b"extra_trailing_bytes", f"unused = {decomp.unused_data!r}"
print("decompressor_unused_data_trailing OK")
"###);
    assert_output(&out, r###"decompressor_unused_data_trailing OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/bz2/higher_level_smaller_or_equal.py`.
#[test]
fn test_gen_behavior_std_libs_bz2_higher_level_smaller_or_equal() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bz2"
# dimension = "behavior"
# case = "higher_level_smaller_or_equal"
# subject = "bz2.compress"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bz2.py"
# status = "filled"
# ///
"""bz2.compress: compresslevel=9 output is no larger than level=1 and still decompresses correctly"""
import bz2

data = b"abcdefghij" * 200
c1 = bz2.compress(data, compresslevel=1)
c9 = bz2.compress(data, compresslevel=9)
assert len(c9) <= len(c1), f"level 9 <= level 1: {len(c9)} vs {len(c1)}"
assert bz2.decompress(c9) == data, "level 9 decompresses correctly"
print("higher_level_smaller_or_equal OK")
"###);
    assert_output(&out, r###"higher_level_smaller_or_equal OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/bz2/incremental_stream_roundtrip.py`.
#[test]
fn test_gen_behavior_std_libs_bz2_incremental_stream_roundtrip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bz2"
# dimension = "behavior"
# case = "incremental_stream_roundtrip"
# subject = "bz2.BZ2Compressor"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bz2.py"
# status = "filled"
# ///
"""bz2.BZ2Compressor: BZ2Compressor.compress chunks + flush concatenate into a stream BZ2Decompressor reassembles, setting eof"""
import bz2

comp = bz2.BZ2Compressor()
parts = [b"chunk1:", b"chunk2:", b"chunk3"]
compressed = b"".join([comp.compress(p) for p in parts]) + comp.flush()
decomp = bz2.BZ2Decompressor()
result = decomp.decompress(compressed)
assert result == b"chunk1:chunk2:chunk3", f"incremental = {result!r}"
assert decomp.eof is True, "eof set after complete decompression"
print("incremental_stream_roundtrip OK")
"###);
    assert_output(&out, r###"incremental_stream_roundtrip OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/bz2/open_bytesio_write_read.py`.
#[test]
fn test_gen_behavior_std_libs_bz2_open_bytesio_write_read() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bz2"
# dimension = "behavior"
# case = "open_bytesio_write_read"
# subject = "bz2.open"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bz2.py"
# status = "filled"
# ///
"""bz2.open: bz2.open on a BytesIO round-trips bytes written in wb mode back through rb mode"""
import bz2
import io

buf = io.BytesIO()
with bz2.open(buf, "wb") as f:
    f.write(b"line 1\n")
    f.write(b"line 2\n")
buf.seek(0)
with bz2.open(buf, "rb") as f:
    content = f.read()
assert content == b"line 1\nline 2\n", f"bz2.open write/read = {content!r}"
print("open_bytesio_write_read OK")
"###);
    assert_output(&out, r###"open_bytesio_write_read OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/bz2/open_encoding_utf16_roundtrip.py`.
#[test]
fn test_gen_behavior_std_libs_bz2_open_encoding_utf16_roundtrip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bz2"
# dimension = "behavior"
# case = "open_encoding_utf16_roundtrip"
# subject = "bz2.open"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bz2.py"
# status = "filled"
# ///
"""bz2.open: a non-default encoding (utf-16-le) survives a bz2.open text write/read round-trip"""
import bz2
import os
import tempfile

with tempfile.TemporaryDirectory() as td:
    fn = os.path.join(td, "f.bz2")
    text = "héllo wörld\nsecond line\n"
    with bz2.open(fn, "wt", encoding="utf-16-le") as f:
        f.write(text)
    with bz2.open(fn, "rt", encoding="utf-16-le") as f:
        assert f.read() == text, "utf-16-le round-trip"
print("open_encoding_utf16_roundtrip OK")
"###);
    assert_output(&out, r###"open_encoding_utf16_roundtrip OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/bz2/open_errors_ignore_handler.py`.
#[test]
fn test_gen_behavior_std_libs_bz2_open_errors_ignore_handler() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bz2"
# dimension = "behavior"
# case = "open_errors_ignore_handler"
# subject = "bz2.open"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bz2.py"
# status = "filled"
# ///
"""bz2.open: bz2.open rt with errors='ignore' drops undecodable bytes when reading binary content as ascii"""
import bz2
import os
import tempfile

with tempfile.TemporaryDirectory() as td:
    fn = os.path.join(td, "f.bz2")
    with bz2.open(fn, "wb") as f:
        f.write(b"foo\xffbar")
    with bz2.open(fn, "rt", encoding="ascii", errors="ignore") as f:
        assert f.read() == "foobar", "errors=ignore"
print("open_errors_ignore_handler OK")
"###);
    assert_output(&out, r###"open_errors_ignore_handler OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/bz2/open_exclusive_create_mode.py`.
#[test]
fn test_gen_behavior_std_libs_bz2_open_exclusive_create_mode() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bz2"
# dimension = "behavior"
# case = "open_exclusive_create_mode"
# subject = "bz2.open"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bz2.py"
# status = "filled"
# ///
"""bz2.open: bz2.open xb mode creates a file exclusively and a second xb open raises FileExistsError"""
import bz2
import os
import tempfile

with tempfile.TemporaryDirectory() as td:
    fn = os.path.join(td, "f.bz2")
    with bz2.open(fn, "xb") as f:
        f.write(b"data")
    try:
        with bz2.open(fn, "xb"):
            pass
        raise AssertionError("expected FileExistsError on second xb open")
    except FileExistsError:
        pass
print("open_exclusive_create_mode OK")
"###);
    assert_output(&out, r###"open_exclusive_create_mode OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/bz2/open_explicit_newline.py`.
#[test]
fn test_gen_behavior_std_libs_bz2_open_explicit_newline() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bz2"
# dimension = "behavior"
# case = "open_explicit_newline"
# subject = "bz2.open"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bz2.py"
# status = "filled"
# ///
"""bz2.open: an explicit newline argument disables universal-newline translation on a text read"""
import bz2
import os
import tempfile

with tempfile.TemporaryDirectory() as td:
    fn = os.path.join(td, "f.bz2")
    plain = "a\nb\nc\n"
    with bz2.open(fn, "wt", encoding="utf-8", newline="\n") as f:
        f.write(plain)
    with bz2.open(fn, "rt", encoding="utf-8", newline="\r") as f:
        assert f.readlines() == [plain], "explicit newline"
print("open_explicit_newline OK")
"###);
    assert_output(&out, r###"open_explicit_newline OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/bz2/open_text_exclusive_roundtrip.py`.
#[test]
fn test_gen_behavior_std_libs_bz2_open_text_exclusive_roundtrip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bz2"
# dimension = "behavior"
# case = "open_text_exclusive_roundtrip"
# subject = "bz2.open"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bz2.py"
# status = "filled"
# ///
"""bz2.open: bz2.open xt mode creates a text file exclusively and rt reads back the written string"""
import bz2
import os
import tempfile

with tempfile.TemporaryDirectory() as td:
    xt = os.path.join(td, "g.bz2")
    with bz2.open(xt, "xt", encoding="utf-8") as f:
        f.write("hi")
    with bz2.open(xt, "rt", encoding="utf-8") as f:
        assert f.read() == "hi", "xt round-trip"
print("open_text_exclusive_roundtrip OK")
"###);
    assert_output(&out, r###"open_text_exclusive_roundtrip OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/bz2/open_text_mode_encoding.py`.
#[test]
fn test_gen_behavior_std_libs_bz2_open_text_mode_encoding() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bz2"
# dimension = "behavior"
# case = "open_text_mode_encoding"
# subject = "bz2.open"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bz2.py"
# status = "filled"
# ///
"""bz2.open: bz2.open in wt/rt mode encodes and decodes unicode text with an explicit encoding"""
import bz2
import io

buf = io.BytesIO()
with bz2.open(buf, "wt", encoding="utf-8") as f:
    f.write("héllo wörld")
buf.seek(0)
with bz2.open(buf, "rt", encoding="utf-8") as f:
    text = f.read()
assert text == "héllo wörld", f"text mode = {text!r}"
print("open_text_mode_encoding OK")
"###);
    assert_output(&out, r###"open_text_mode_encoding OK
"###);
}
