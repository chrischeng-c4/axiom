use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/io/binary_readline_size_caps_bytes.py`.
#[test]
fn test_gen_behavior_std_libs_io_binary_readline_size_caps_bytes() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "io"
# dimension = "behavior"
# case = "binary_readline_size_caps_bytes"
# subject = "io.BufferedReader"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_io.py"
# status = "filled"
# ///
"""io.BufferedReader: readline(size) on a binary stream caps the returned bytes per call, handles NUL bytes, None reads a whole line, and a float size raises TypeError"""
import io

import os
import tempfile

with tempfile.TemporaryDirectory() as d:
    path = os.path.join(d, "data.txt")
    with open(path, "wb") as f:
        f.write(b"abc\ndef\nxyzzy\nfoo\x00bar\ntail")
    with open(path, "rb") as f:
        assert f.readline() == b"abc\n", "full line"
        assert f.readline(10) == b"def\n", "size beyond newline"
        assert f.readline(2) == b"xy", "size mid-line"
        assert f.readline(4) == b"zzy\n", "size to newline"
        assert f.readline() == b"foo\x00bar\n", "line with NUL byte"
        assert f.readline(None) == b"tail", "None size reads whole line"
        float_size = False
        try:
            f.readline(5.3)
        except TypeError:
            float_size = True
        assert float_size, "float readline size did not raise TypeError"

print("binary_readline_size_caps_bytes OK")
"###);
    assert_output(&out, r###"binary_readline_size_caps_bytes OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/io/bytesio_getvalue_returns_bytes.py`.
#[test]
fn test_gen_behavior_std_libs_io_bytesio_getvalue_returns_bytes() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "io"
# dimension = "behavior"
# case = "bytesio_getvalue_returns_bytes"
# subject = "io.BytesIO"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""io.BytesIO: BytesIO.getvalue() returns a bytes object equal to the written content"""
import io

_bv = io.BytesIO(b"\x01\x02")
assert isinstance(_bv.getvalue(), bytes), "getvalue returns bytes"
assert _bv.getvalue() == b"\x01\x02", f"BytesIO getvalue = {_bv.getvalue()!r}"

print("bytesio_getvalue_returns_bytes OK")
"###);
    assert_output(&out, r###"bytesio_getvalue_returns_bytes OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/io/bytesio_write_read_roundtrip.py`.
#[test]
fn test_gen_behavior_std_libs_io_bytesio_write_read_roundtrip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "io"
# dimension = "behavior"
# case = "bytesio_write_read_roundtrip"
# subject = "io.BytesIO"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""io.BytesIO: BytesIO writes and reads bytes; seek(0) then read() returns the written bytes"""
import io

_bbuf = io.BytesIO()
_bbuf.write(b"bytes data")
_bbuf.seek(0)
assert _bbuf.read() == b"bytes data", "BytesIO round-trip"

print("bytesio_write_read_roundtrip OK")
"###);
    assert_output(&out, r###"bytesio_write_read_roundtrip OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/io/open_append_starts_at_end.py`.
#[test]
fn test_gen_behavior_std_libs_io_open_append_starts_at_end() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "io"
# dimension = "behavior"
# case = "open_append_starts_at_end"
# subject = "io.open"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_io.py"
# status = "filled"
# ///
"""io.open: append mode ('ab'/'a') positions the new handle at the end of existing content"""
import io

import os
import tempfile

with tempfile.TemporaryDirectory() as d:
    path = os.path.join(d, "data.txt")
    with open(path, "wb") as f:
        f.write(b"xxx")
    with open(path, "ab", buffering=0) as f:
        assert f.tell() == 3, f"append binary tell = {f.tell()!r}"
    with open(path, "a", encoding="utf-8") as f:
        assert f.tell() > 0, f"append text tell = {f.tell()!r}"

print("open_append_starts_at_end OK")
"###);
    assert_output(&out, r###"open_append_starts_at_end OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/io/open_close_flushes_to_disk.py`.
#[test]
fn test_gen_behavior_std_libs_io_open_close_flushes_to_disk() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "io"
# dimension = "behavior"
# case = "open_close_flushes_to_disk"
# subject = "io.open"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_io.py"
# status = "filled"
# ///
"""io.open: close() flushes buffered writes so a fresh read sees the full content"""
import io

import os
import tempfile

with tempfile.TemporaryDirectory() as d:
    path = os.path.join(d, "data.txt")
    with open(path, "wb") as f:
        f.write(b"flushed")
    with open(path, "rb") as f:
        assert f.read() == b"flushed", "close did not flush"

print("open_close_flushes_to_disk OK")
"###);
    assert_output(&out, r###"open_close_flushes_to_disk OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/io/open_close_is_idempotent.py`.
#[test]
fn test_gen_behavior_std_libs_io_open_close_is_idempotent() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "io"
# dimension = "behavior"
# case = "open_close_is_idempotent"
# subject = "io.open"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_io.py"
# status = "filled"
# ///
"""io.open: close() is idempotent (repeated calls are safe) but flush() on a closed file raises ValueError"""
import io

import os
import tempfile

with tempfile.TemporaryDirectory() as d:
    path = os.path.join(d, "data.txt")
    f = open(path, "wb", buffering=0)
    f.close()
    f.close()
    f.close()
    flush_raised = False
    try:
        f.flush()
    except ValueError:
        flush_raised = True
    assert flush_raised, "flush on closed file did not raise ValueError"

print("open_close_is_idempotent OK")
"###);
    assert_output(&out, r###"open_close_is_idempotent OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/io/open_closefd_true_for_filename.py`.
#[test]
fn test_gen_behavior_std_libs_io_open_closefd_true_for_filename() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "io"
# dimension = "behavior"
# case = "open_closefd_true_for_filename"
# subject = "io.open"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_io.py"
# status = "filled"
# ///
"""io.open: a filename open has buffer.raw.closefd True; a borrowed-fd open (closefd=False) has it False and reads the same content"""
import io

import os
import tempfile

with tempfile.TemporaryDirectory() as d:
    path = os.path.join(d, "data.txt")
    with open(path, "w", encoding="utf-8") as f:
        f.write("egg\n")
    with open(path, "r", encoding="utf-8") as f:
        assert f.buffer.raw.closefd is True, "filename open closefd"
        borrowed = open(f.fileno(), "r", encoding="utf-8", closefd=False)
        assert borrowed.buffer.raw.closefd is False, "borrowed fd closefd"
        assert borrowed.read() == "egg\n", "read via borrowed fd"
        borrowed.close()
        read_after_close = False
        try:
            borrowed.read()
        except ValueError:
            read_after_close = True
        assert read_after_close, "read after close did not raise ValueError"

print("open_closefd_true_for_filename OK")
"###);
    assert_output(&out, r###"open_closefd_true_for_filename OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/io/open_exclusive_create_writes_new.py`.
#[test]
fn test_gen_behavior_std_libs_io_open_exclusive_create_writes_new() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "io"
# dimension = "behavior"
# case = "open_exclusive_create_writes_new"
# subject = "io.open"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_io.py"
# status = "filled"
# ///
"""io.open: exclusive-create mode 'xb' writes a brand-new file and round-trips its content"""
import io

import os
import tempfile

with tempfile.TemporaryDirectory() as d:
    new = os.path.join(d, "new.bin")
    with open(new, "xb") as f:
        f.write(b"spam")
    with open(new, "rb") as f:
        assert f.read() == b"spam", "xb create round-trip"

print("open_exclusive_create_writes_new OK")
"###);
    assert_output(&out, r###"open_exclusive_create_writes_new OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/io/open_opener_supplies_fd.py`.
#[test]
fn test_gen_behavior_std_libs_io_open_opener_supplies_fd() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "io"
# dimension = "behavior"
# case = "open_opener_supplies_fd"
# subject = "io.open"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_io.py"
# status = "filled"
# ///
"""io.open: an opener callback supplies the underlying fd and the filename argument is ignored"""
import io

import os
import tempfile

with tempfile.TemporaryDirectory() as d:
    path = os.path.join(d, "data.txt")
    with open(path, "wb") as f:
        f.write(b"abcdef")
    fd = os.open(path, os.O_RDONLY)
    with open("ignored", "rb", opener=lambda p, flags: fd) as f:
        assert f.read()[:3] == b"abc", "opener-supplied fd read"

print("open_opener_supplies_fd OK")
"###);
    assert_output(&out, r###"open_opener_supplies_fd OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/io/open_with_autocloses_on_error.py`.
#[test]
fn test_gen_behavior_std_libs_io_open_with_autocloses_on_error() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "io"
# dimension = "behavior"
# case = "open_with_autocloses_on_error"
# subject = "io.open"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_io.py"
# status = "filled"
# ///
"""io.open: a 'with open(...)' block auto-closes the file even when the body raises, across buffer sizes"""
import io

import os
import tempfile

with tempfile.TemporaryDirectory() as d:
    path = os.path.join(d, "data.txt")
    for bufsize in (0, 100):
        raised = False
        try:
            with open(path, "wb", bufsize) as f:
                raise ZeroDivisionError
        except ZeroDivisionError:
            raised = True
        assert raised, "exception swallowed"
        assert f.closed, f"not closed after error (bufsize={bufsize})"

print("open_with_autocloses_on_error OK")
"###);
    assert_output(&out, r###"open_with_autocloses_on_error OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/io/open_with_autocloses_on_exit.py`.
#[test]
fn test_gen_behavior_std_libs_io_open_with_autocloses_on_exit() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "io"
# dimension = "behavior"
# case = "open_with_autocloses_on_exit"
# subject = "io.open"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_io.py"
# status = "filled"
# ///
"""io.open: a 'with open(...)' block auto-closes the file on normal exit (.closed becomes True), across buffer sizes"""
import io

import os
import tempfile

with tempfile.TemporaryDirectory() as d:
    path = os.path.join(d, "data.txt")
    for bufsize in (0, 100):
        with open(path, "wb", bufsize) as f:
            f.write(b"xxx")
        assert f.closed, f"not closed after with (bufsize={bufsize})"

print("open_with_autocloses_on_exit OK")
"###);
    assert_output(&out, r###"open_with_autocloses_on_exit OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/io/seek_constants_values.py`.
#[test]
fn test_gen_behavior_std_libs_io_seek_constants_values() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "io"
# dimension = "behavior"
# case = "seek_constants_values"
# subject = "io.SEEK_SET"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""io.SEEK_SET: the seek-origin constants are SEEK_SET == 0, SEEK_CUR == 1, SEEK_END == 2"""
import io

assert io.SEEK_SET == 0, f"SEEK_SET = {io.SEEK_SET!r}"
assert io.SEEK_CUR == 1, f"SEEK_CUR = {io.SEEK_CUR!r}"
assert io.SEEK_END == 2, f"SEEK_END = {io.SEEK_END!r}"

print("seek_constants_values OK")
"###);
    assert_output(&out, r###"seek_constants_values OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/io/stringio_close_marks_closed.py`.
#[test]
fn test_gen_behavior_std_libs_io_stringio_close_marks_closed() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "io"
# dimension = "behavior"
# case = "stringio_close_marks_closed"
# subject = "io.StringIO"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""io.StringIO: close() sets .closed True and a subsequent read() raises ValueError"""
import io

_c = io.StringIO("x")
_c.close()
assert _c.closed, "closed after close()"
_raised = False
try:
    _c.read()
except ValueError:
    _raised = True
assert _raised, "read on closed raises ValueError"

print("stringio_close_marks_closed OK")
"###);
    assert_output(&out, r###"stringio_close_marks_closed OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/io/stringio_getvalue_ignores_position.py`.
#[test]
fn test_gen_behavior_std_libs_io_stringio_getvalue_ignores_position() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "io"
# dimension = "behavior"
# case = "stringio_getvalue_ignores_position"
# subject = "io.StringIO"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""io.StringIO: getvalue() returns the entire buffer regardless of the current read/write position"""
import io

_buf = io.StringIO()
_buf.write("full")
_buf.seek(0)
_buf.read(2)
assert _buf.getvalue() == "full", f"getvalue mid-read = {_buf.getvalue()!r}"

print("stringio_getvalue_ignores_position OK")
"###);
    assert_output(&out, r###"stringio_getvalue_ignores_position OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/io/stringio_getvalue_without_seek.py`.
#[test]
fn test_gen_behavior_std_libs_io_stringio_getvalue_without_seek() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "io"
# dimension = "behavior"
# case = "stringio_getvalue_without_seek"
# subject = "io.StringIO"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""io.StringIO: getvalue() returns written content without needing a prior seek(0)"""
import io

_buf = io.StringIO()
_buf.write("abc")
assert _buf.getvalue() == "abc", f"getvalue = {_buf.getvalue()!r}"

print("stringio_getvalue_without_seek OK")
"###);
    assert_output(&out, r###"stringio_getvalue_without_seek OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/io/stringio_iteration_yields_lines.py`.
#[test]
fn test_gen_behavior_std_libs_io_stringio_iteration_yields_lines() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "io"
# dimension = "behavior"
# case = "stringio_iteration_yields_lines"
# subject = "io.StringIO"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""io.StringIO: iterating a StringIO yields each line (newline-terminated) in order"""
import io

_lines_buf = io.StringIO("line1\nline2\nline3\n")
_lines = list(_lines_buf)
assert _lines == ["line1\n", "line2\n", "line3\n"], f"iteration lines = {_lines!r}"

print("stringio_iteration_yields_lines OK")
"###);
    assert_output(&out, r###"stringio_iteration_yields_lines OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/io/stringio_read_n_caps_chars.py`.
#[test]
fn test_gen_behavior_std_libs_io_stringio_read_n_caps_chars() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "io"
# dimension = "behavior"
# case = "stringio_read_n_caps_chars"
# subject = "io.StringIO"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""io.StringIO: read(n) returns at most n characters from the current position"""
import io

_buf = io.StringIO("hello world")
assert _buf.read(5) == "hello", "read(5) caps at 5 chars"

print("stringio_read_n_caps_chars OK")
"###);
    assert_output(&out, r###"stringio_read_n_caps_chars OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/io/stringio_readable_writable_seekable.py`.
#[test]
fn test_gen_behavior_std_libs_io_stringio_readable_writable_seekable() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "io"
# dimension = "behavior"
# case = "stringio_readable_writable_seekable"
# subject = "io.StringIO"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""io.StringIO: a StringIO reports readable(), writable(), and seekable() all True"""
import io

_r = io.StringIO("x")
assert _r.readable(), "StringIO readable"
assert _r.writable(), "StringIO writable"
assert _r.seekable(), "StringIO seekable"

print("stringio_readable_writable_seekable OK")
"###);
    assert_output(&out, r###"stringio_readable_writable_seekable OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/io/stringio_readline_yields_line.py`.
#[test]
fn test_gen_behavior_std_libs_io_stringio_readline_yields_line() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "io"
# dimension = "behavior"
# case = "stringio_readline_yields_line"
# subject = "io.StringIO"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""io.StringIO: readline() returns the next line including its trailing newline"""
import io

_lines = io.StringIO("line1\nline2\nline3")
assert _lines.readline() == "line1\n", "readline keeps trailing newline"

print("stringio_readline_yields_line OK")
"###);
    assert_output(&out, r###"stringio_readline_yields_line OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/io/stringio_seek0_read_full.py`.
#[test]
fn test_gen_behavior_std_libs_io_stringio_seek0_read_full() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "io"
# dimension = "behavior"
# case = "stringio_seek0_read_full"
# subject = "io.StringIO"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""io.StringIO: after writing then seek(0), read() returns the full buffer contents"""
import io

_buf = io.StringIO()
_buf.write("abcdef")
_buf.seek(0)
assert _buf.read() == "abcdef", "read after seek(0)"

print("stringio_seek0_read_full OK")
"###);
    assert_output(&out, r###"stringio_seek0_read_full OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/io/stringio_seek_end_positions_at_end.py`.
#[test]
fn test_gen_behavior_std_libs_io_stringio_seek_end_positions_at_end() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "io"
# dimension = "behavior"
# case = "stringio_seek_end_positions_at_end"
# subject = "io.StringIO"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""io.StringIO: seek(0, SEEK_END) returns and positions at the byte/char length of the buffer"""
import io

_end = io.StringIO("abc")
_pos = _end.seek(0, io.SEEK_END)
assert _pos == 3, f"seek SEEK_END = {_pos!r}"

print("stringio_seek_end_positions_at_end OK")
"###);
    assert_output(&out, r###"stringio_seek_end_positions_at_end OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/io/stringio_tell_tracks_position.py`.
#[test]
fn test_gen_behavior_std_libs_io_stringio_tell_tracks_position() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "io"
# dimension = "behavior"
# case = "stringio_tell_tracks_position"
# subject = "io.StringIO"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""io.StringIO: tell() starts at 0, advances by characters read, and returns to 0 after seek(0)"""
import io

_t = io.StringIO("hello")
assert _t.tell() == 0, f"initial tell = {_t.tell()!r}"
_t.read(3)
assert _t.tell() == 3, f"after read(3) tell = {_t.tell()!r}"
_t.seek(0)
assert _t.tell() == 0, f"after seek(0) tell = {_t.tell()!r}"

print("stringio_tell_tracks_position OK")
"###);
    assert_output(&out, r###"stringio_tell_tracks_position OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/io/stringio_truncate_shrinks.py`.
#[test]
fn test_gen_behavior_std_libs_io_stringio_truncate_shrinks() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "io"
# dimension = "behavior"
# case = "stringio_truncate_shrinks"
# subject = "io.StringIO"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""io.StringIO: truncate() at the current position drops everything after it"""
import io

_trunc = io.StringIO("hello world")
_trunc.seek(5)
_trunc.truncate()
_trunc.seek(0)
assert _trunc.read() == "hello", "truncate drops everything after position"

print("stringio_truncate_shrinks OK")
"###);
    assert_output(&out, r###"stringio_truncate_shrinks OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/io/stringio_write_returns_char_count.py`.
#[test]
fn test_gen_behavior_std_libs_io_stringio_write_returns_char_count() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "io"
# dimension = "behavior"
# case = "stringio_write_returns_char_count"
# subject = "io.StringIO"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""io.StringIO: StringIO.write returns the number of characters written (write('hello') -> 5)"""
import io

_buf = io.StringIO()
_n = _buf.write("hello")
assert _n == 5, f"write returns char count = {_n!r}"

print("stringio_write_returns_char_count OK")
"###);
    assert_output(&out, r###"stringio_write_returns_char_count OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/io/text_bom_written_once.py`.
#[test]
fn test_gen_behavior_std_libs_io_text_bom_written_once() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "io"
# dimension = "behavior"
# case = "text_bom_written_once"
# subject = "io.TextIOWrapper"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_io.py"
# status = "filled"
# ///
"""io.TextIOWrapper: BOM-bearing encodings (utf-8-sig/utf-16/utf-32) write the BOM exactly once; appending does not re-emit it"""
import io

import os
import tempfile

with tempfile.TemporaryDirectory() as d:
    path = os.path.join(d, "data.txt")
    for charset in ("utf-8-sig", "utf-16", "utf-32"):
        with open(path, "w", encoding=charset) as f:
            f.write("aaa")
        with open(path, "rb") as f:
            assert f.read() == "aaa".encode(charset), f"BOM write {charset}"
        with open(path, "a", encoding=charset) as f:
            f.write("xxx")
        with open(path, "rb") as f:
            assert f.read() == "aaaxxx".encode(charset), f"append no re-BOM {charset}"

print("text_bom_written_once OK")
"###);
    assert_output(&out, r###"text_bom_written_once OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/io/text_multibyte_tell_seek_stable.py`.
#[test]
fn test_gen_behavior_std_libs_io_text_multibyte_tell_seek_stable() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "io"
# dimension = "behavior"
# case = "text_multibyte_tell_seek_stable"
# subject = "io.TextIOWrapper"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_io.py"
# status = "filled"
# ///
"""io.TextIOWrapper: for a multibyte encoding (euc_jp), tell() positions are stable across seek/readline round-trips"""
import io

import os
import tempfile

with tempfile.TemporaryDirectory() as d:
    path = os.path.join(d, "data.txt")
    with open(path, "w", encoding="euc_jp") as f:
        f.write("AB\nうえ\n")
    with open(path, "r", encoding="euc_jp") as f:
        assert f.readline() == "AB\n", "euc_jp line 1"
        p0 = f.tell()
        assert f.readline() == "うえ\n", "euc_jp line 2"
        p1 = f.tell()
        f.seek(p0)
        assert f.readline() == "うえ\n", "euc_jp re-read after seek"
        assert f.tell() == p1, "euc_jp tell after re-read"

print("text_multibyte_tell_seek_stable OK")
"###);
    assert_output(&out, r###"text_multibyte_tell_seek_stable OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/io/text_seek_rewrite_preserves_single_bom.py`.
#[test]
fn test_gen_behavior_std_libs_io_text_seek_rewrite_preserves_single_bom() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "io"
# dimension = "behavior"
# case = "text_seek_rewrite_preserves_single_bom"
# subject = "io.TextIOWrapper"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_io.py"
# status = "filled"
# ///
"""io.TextIOWrapper: seeking past the BOM, then rewriting from the start, preserves exactly one leading BOM"""
import io

import os
import tempfile

with tempfile.TemporaryDirectory() as d:
    path = os.path.join(d, "data.txt")
    for charset in ("utf-8-sig", "utf-16", "utf-32"):
        with open(path, "w", encoding=charset) as f:
            f.write("aaa")
            pos = f.tell()
        with open(path, "r+", encoding=charset) as f:
            f.seek(pos)
            f.write("zzz")
            f.seek(0)
            f.write("bbb")
        with open(path, "rb") as f:
            assert f.read() == "bbbzzz".encode(charset), f"seek-rewrite {charset}"

print("text_seek_rewrite_preserves_single_bom OK")
"###);
    assert_output(&out, r###"text_seek_rewrite_preserves_single_bom OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/io/text_tell_roundtrip_rewinds_lines.py`.
#[test]
fn test_gen_behavior_std_libs_io_text_tell_roundtrip_rewinds_lines() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "io"
# dimension = "behavior"
# case = "text_tell_roundtrip_rewinds_lines"
# subject = "io.TextIOWrapper"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_io.py"
# status = "filled"
# ///
"""io.TextIOWrapper: each tell()-recorded position rewinds to exactly the right line on seek (utf-8 multi-line round-trip)"""
import io

import os
import tempfile

with tempfile.TemporaryDirectory() as d:
    path = os.path.join(d, "data.txt")
    with open(path, "w+", encoding="utf-8") as f:
        p0 = f.tell()
        f.write("ÿ\n")
        p1 = f.tell()
        f.write("ÿ\n")
        p2 = f.tell()
        f.seek(0)
        assert f.tell() == p0, "start position"
        assert f.readline() == "ÿ\n", "line 1"
        assert f.tell() == p1, "after line 1"
        assert f.readline() == "ÿ\n", "line 2"
        assert f.tell() == p2, "after line 2"

print("text_tell_roundtrip_rewinds_lines OK")
"###);
    assert_output(&out, r###"text_tell_roundtrip_rewinds_lines OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/io/textwrapper_errors_handler_reflected.py`.
#[test]
fn test_gen_behavior_std_libs_io_textwrapper_errors_handler_reflected() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "io"
# dimension = "behavior"
# case = "textwrapper_errors_handler_reflected"
# subject = "io.TextIOWrapper"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_io.py"
# status = "filled"
# ///
"""io.TextIOWrapper: TextIOWrapper.errors reflects the error handler chosen at open time (default 'strict', or 'replace')"""
import io

import os
import tempfile

with tempfile.TemporaryDirectory() as d:
    path = os.path.join(d, "data.txt")
    with open(path, "w", encoding="utf-8") as f:
        assert f.errors == "strict", f"default errors = {f.errors!r}"
    with open(path, "w", encoding="utf-8", errors="replace") as f:
        assert f.errors == "replace", f"errors = {f.errors!r}"

print("textwrapper_errors_handler_reflected OK")
"###);
    assert_output(&out, r###"textwrapper_errors_handler_reflected OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/io/textwrapper_exposes_buffer_layers.py`.
#[test]
fn test_gen_behavior_std_libs_io_textwrapper_exposes_buffer_layers() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "io"
# dimension = "behavior"
# case = "textwrapper_exposes_buffer_layers"
# subject = "io.TextIOWrapper"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_io.py"
# status = "filled"
# ///
"""io.TextIOWrapper: a text-mode handle exposes .mode, .buffer.mode and .buffer.raw.mode reflecting the layered stack"""
import io

import os
import tempfile

with tempfile.TemporaryDirectory() as d:
    path = os.path.join(d, "data.txt")
    with open(path, "w+", encoding="utf-8") as f:
        assert f.mode == "w+", f"text mode = {f.mode!r}"
        assert f.buffer.mode == "rb+", f"buffer mode = {f.buffer.mode!r}"
        assert f.buffer.raw.mode == "rb+", f"raw mode = {f.buffer.raw.mode!r}"

print("textwrapper_exposes_buffer_layers OK")
"###);
    assert_output(&out, r###"textwrapper_exposes_buffer_layers OK
"###);
}
