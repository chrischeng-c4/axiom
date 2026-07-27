use super::super::super::super::harness::*;

/// Ported from `tests/cpython/errors/std-libs/io/closed_file_ops_raise_value_error.py`.
#[test]
fn test_gen_errors_std_libs_io_closed_file_ops_raise_value_error() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "io"
# dimension = "errors"
# case = "closed_file_ops_raise_value_error"
# subject = "io"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_io.py"
# status = "filled"
# ///
"""io: every operation (flush/read/write/seek/tell/truncate/next/...) on a closed file object raises ValueError, across text, buffered and raw layers"""
import io

import os
import tempfile

MODES = [
    {"mode": "w"}, {"mode": "wb"},
    {"mode": "w", "buffering": 1}, {"mode": "wb", "buffering": 0},
    {"mode": "r"}, {"mode": "rb"}, {"mode": "rb", "buffering": 0},
    {"mode": "w+"}, {"mode": "w+b"}, {"mode": "w+b", "buffering": 0},
]


def expect_value_error(fn, what):
    try:
        fn()
    except ValueError:
        return
    raise AssertionError(f"{what}: expected ValueError on closed stream")


with tempfile.TemporaryDirectory() as d:
    path = os.path.join(d, "data.txt")
    for kwargs in MODES:
        binary = "b" in kwargs["mode"]
        if not binary:
            kwargs = {**kwargs, "encoding": "utf-8"}
        f = open(path, **kwargs)
        f.close()

        expect_value_error(f.flush, "flush")
        expect_value_error(f.fileno, "fileno")
        expect_value_error(f.isatty, "isatty")
        expect_value_error(f.__iter__, "__iter__")
        expect_value_error(f.read, "read")
        expect_value_error(f.readline, "readline")
        expect_value_error(f.readlines, "readlines")
        expect_value_error(lambda: f.seek(0), "seek")
        expect_value_error(f.tell, "tell")
        expect_value_error(f.truncate, "truncate")
        expect_value_error(lambda: f.write(b"" if binary else ""), "write")
        expect_value_error(lambda: f.writelines([]), "writelines")
        expect_value_error(lambda: next(f), "next")
        if hasattr(f, "peek"):
            expect_value_error(lambda: f.peek(1), "peek")
        if hasattr(f, "readinto"):
            expect_value_error(lambda: f.readinto(bytearray(8)), "readinto")

print("closed_file_ops_raise_value_error OK")
"###);
    assert_output(&out, r###"closed_file_ops_raise_value_error OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/io/open_closefd_false_filename_raises.py`.
#[test]
fn test_gen_errors_std_libs_io_open_closefd_false_filename_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "io"
# dimension = "errors"
# case = "open_closefd_false_filename_raises"
# subject = "io.open"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_io.py"
# status = "filled"
# ///
"""io.open: open(path, closefd=False) raises ValueError because closefd=False is only valid for an existing fd, not a filename"""
import io

import os
import tempfile

with tempfile.TemporaryDirectory() as d:
    path = os.path.join(d, "data.txt")
    for mode in ("w", "r"):
        if mode == "r":
            with open(path, "w", encoding="utf-8"):
                pass
        raised = False
        try:
            open(path, mode, encoding="utf-8", closefd=False)
        except ValueError:
            raised = True
        assert raised, f"closefd=False on filename ({mode}) must raise ValueError"

print("open_closefd_false_filename_raises OK")
"###);
    assert_output(&out, r###"open_closefd_false_filename_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/io/open_exclusive_existing_raises.py`.
#[test]
fn test_gen_errors_std_libs_io_open_exclusive_existing_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "io"
# dimension = "errors"
# case = "open_exclusive_existing_raises"
# subject = "io.open"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_io.py"
# status = "filled"
# ///
"""io.open: open(path, 'x') on an already-existing file raises FileExistsError"""
import io

import os
import tempfile

with tempfile.TemporaryDirectory() as d:
    path = os.path.join(d, "data.txt")
    with open(path, "w", encoding="utf-8"):
        pass
    raised = False
    try:
        open(path, "x", encoding="utf-8")
    except FileExistsError:
        raised = True
    assert raised, "exclusive-create on existing file must raise FileExistsError"

print("open_exclusive_existing_raises OK")
"###);
    assert_output(&out, r###"open_exclusive_existing_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/io/open_invalid_mode_raises.py`.
#[test]
fn test_gen_errors_std_libs_io_open_invalid_mode_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "io"
# dimension = "errors"
# case = "open_invalid_mode_raises"
# subject = "io.open"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_io.py"
# status = "filled"
# ///
"""io.open: open() with a nonsensical mode string ('rwax+') raises ValueError"""
import io

import os
import tempfile

with tempfile.TemporaryDirectory() as d:
    path = os.path.join(d, "data.txt")
    raised = False
    try:
        open(path, "rwax+", encoding="utf-8")
    except ValueError:
        raised = True
    assert raised, "nonsensical mode string must raise ValueError"

print("open_invalid_mode_raises OK")
"###);
    assert_output(&out, r###"open_invalid_mode_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/io/open_nul_in_path_raises.py`.
#[test]
fn test_gen_errors_std_libs_io_open_nul_in_path_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "io"
# dimension = "errors"
# case = "open_nul_in_path_raises"
# subject = "io.open"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_io.py"
# status = "filled"
# ///
"""io.open: open() of a path containing an embedded NUL byte raises ValueError"""
import io

raised = False
try:
    open("foo\x00bar", "w", encoding="utf-8")
except ValueError:
    raised = True
assert raised, "embedded NUL byte in path must raise ValueError"

print("open_nul_in_path_raises OK")
"###);
    assert_output(&out, r###"open_nul_in_path_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/io/read_closed_bytesio_raises.py`.
#[test]
fn test_gen_errors_std_libs_io_read_closed_bytesio_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "io"
# dimension = "errors"
# case = "read_closed_bytesio_raises"
# subject = "io.BytesIO"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_io.py"
# status = "filled"
# ///
"""io.BytesIO: read_closed_bytesio_raises (errors)."""
import io

_raised = False
try:
    _b = io.BytesIO(b'hello'); _b.close(); _b.read()
except ValueError:
    _raised = True
assert _raised, "read_closed_bytesio_raises: expected ValueError"
print("read_closed_bytesio_raises OK")
"###);
    assert_output(&out, r###"read_closed_bytesio_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/io/read_closed_stringio_raises.py`.
#[test]
fn test_gen_errors_std_libs_io_read_closed_stringio_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "io"
# dimension = "errors"
# case = "read_closed_stringio_raises"
# subject = "io.StringIO"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_io.py"
# status = "filled"
# ///
"""io.StringIO: read_closed_stringio_raises (errors)."""
import io

_raised = False
try:
    _s = io.StringIO('text'); _s.close(); _s.read()
except ValueError:
    _raised = True
assert _raised, "read_closed_stringio_raises: expected ValueError"
print("read_closed_stringio_raises OK")
"###);
    assert_output(&out, r###"read_closed_stringio_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/io/read_on_write_only_raises.py`.
#[test]
fn test_gen_errors_std_libs_io_read_on_write_only_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "io"
# dimension = "errors"
# case = "read_on_write_only_raises"
# subject = "io.BufferedWriter"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_io.py"
# status = "filled"
# ///
"""io.BufferedWriter: read_on_write_only_raises (errors)."""
import io

_raised = False
try:
    io.BufferedWriter(io.BytesIO()).read(5)
except io.UnsupportedOperation:
    _raised = True
assert _raised, "read_on_write_only_raises: expected io.UnsupportedOperation"
print("read_on_write_only_raises OK")
"###);
    assert_output(&out, r###"read_on_write_only_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/io/seek_bad_whence_raises.py`.
#[test]
fn test_gen_errors_std_libs_io_seek_bad_whence_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "io"
# dimension = "errors"
# case = "seek_bad_whence_raises"
# subject = "io.BytesIO"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_io.py"
# status = "filled"
# ///
"""io.BytesIO: seek_bad_whence_raises (errors)."""
import io

_raised = False
try:
    io.BytesIO(b'data').seek(0, 99)
except ValueError:
    _raised = True
assert _raised, "seek_bad_whence_raises: expected ValueError"
print("seek_bad_whence_raises OK")
"###);
    assert_output(&out, r###"seek_bad_whence_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/io/seek_negative_set_raises.py`.
#[test]
fn test_gen_errors_std_libs_io_seek_negative_set_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "io"
# dimension = "errors"
# case = "seek_negative_set_raises"
# subject = "io.BytesIO"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_io.py"
# status = "filled"
# ///
"""io.BytesIO: seek_negative_set_raises (errors)."""
import io

_raised = False
try:
    io.BytesIO(b'data').seek(-1, 0)
except ValueError:
    _raised = True
assert _raised, "seek_negative_set_raises: expected ValueError"
print("seek_negative_set_raises OK")
"###);
    assert_output(&out, r###"seek_negative_set_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/io/write_bytes_to_stringio_raises.py`.
#[test]
fn test_gen_errors_std_libs_io_write_bytes_to_stringio_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "io"
# dimension = "errors"
# case = "write_bytes_to_stringio_raises"
# subject = "io.StringIO"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_io.py"
# status = "filled"
# ///
"""io.StringIO: write_bytes_to_stringio_raises (errors)."""
import io

_raised = False
try:
    io.StringIO().write(b'bytes')
except TypeError:
    _raised = True
assert _raised, "write_bytes_to_stringio_raises: expected TypeError"
print("write_bytes_to_stringio_raises OK")
"###);
    assert_output(&out, r###"write_bytes_to_stringio_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/io/write_str_to_bytesio_raises.py`.
#[test]
fn test_gen_errors_std_libs_io_write_str_to_bytesio_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "io"
# dimension = "errors"
# case = "write_str_to_bytesio_raises"
# subject = "io.BytesIO"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_io.py"
# status = "filled"
# ///
"""io.BytesIO: write_str_to_bytesio_raises (errors)."""
import io

_raised = False
try:
    io.BytesIO().write('str')
except TypeError:
    _raised = True
assert _raised, "write_str_to_bytesio_raises: expected TypeError"
print("write_str_to_bytesio_raises OK")
"###);
    assert_output(&out, r###"write_str_to_bytesio_raises OK
"###);
}
