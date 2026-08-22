use super::super::super::super::harness::*;

/// Ported from `tests/cpython/errors/std-libs/bz2/bz2file_compresslevel_ten_raises.py`.
#[test]
fn test_gen_errors_std_libs_bz2_bz2file_compresslevel_ten_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bz2"
# dimension = "errors"
# case = "bz2file_compresslevel_ten_raises"
# subject = "bz2.BZ2File"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bz2.py"
# status = "filled"
# ///
"""bz2.BZ2File: bz2file_compresslevel_ten_raises (errors)."""
import bz2, io

_raised = False
try:
    bz2.BZ2File(io.BytesIO(), "w", compresslevel=10)
except ValueError:
    _raised = True
assert _raised, "bz2file_compresslevel_ten_raises: expected ValueError"
print("bz2file_compresslevel_ten_raises OK")
"###);
    assert_output(&out, r###"bz2file_compresslevel_ten_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/bz2/bz2file_compresslevel_zero_raises.py`.
#[test]
fn test_gen_errors_std_libs_bz2_bz2file_compresslevel_zero_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bz2"
# dimension = "errors"
# case = "bz2file_compresslevel_zero_raises"
# subject = "bz2.BZ2File"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bz2.py"
# status = "filled"
# ///
"""bz2.BZ2File: bz2file_compresslevel_zero_raises (errors)."""
import bz2, io

_raised = False
try:
    bz2.BZ2File(io.BytesIO(), "w", compresslevel=0)
except ValueError:
    _raised = True
assert _raised, "bz2file_compresslevel_zero_raises: expected ValueError"
print("bz2file_compresslevel_zero_raises OK")
"###);
    assert_output(&out, r###"bz2file_compresslevel_zero_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/bz2/bz2file_float_filename_raises.py`.
#[test]
fn test_gen_errors_std_libs_bz2_bz2file_float_filename_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bz2"
# dimension = "errors"
# case = "bz2file_float_filename_raises"
# subject = "bz2.BZ2File"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bz2.py"
# status = "filled"
# ///
"""bz2.BZ2File: bz2file_float_filename_raises (errors)."""
import bz2

_raised = False
try:
    bz2.BZ2File(123.456)
except TypeError:
    _raised = True
assert _raised, "bz2file_float_filename_raises: expected TypeError"
print("bz2file_float_filename_raises OK")
"###);
    assert_output(&out, r###"bz2file_float_filename_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/bz2/compress_level_ten_raises.py`.
#[test]
fn test_gen_errors_std_libs_bz2_compress_level_ten_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bz2"
# dimension = "errors"
# case = "compress_level_ten_raises"
# subject = "bz2.compress"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bz2.py"
# status = "filled"
# ///
"""bz2.compress: compress_level_ten_raises (errors)."""
import bz2

_raised = False
try:
    bz2.compress(b"x", 10)
except ValueError:
    _raised = True
assert _raised, "compress_level_ten_raises: expected ValueError"
print("compress_level_ten_raises OK")
"###);
    assert_output(&out, r###"compress_level_ten_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/bz2/compress_level_zero_raises.py`.
#[test]
fn test_gen_errors_std_libs_bz2_compress_level_zero_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bz2"
# dimension = "errors"
# case = "compress_level_zero_raises"
# subject = "bz2.compress"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bz2.py"
# status = "filled"
# ///
"""bz2.compress: compress_level_zero_raises (errors)."""
import bz2

_raised = False
try:
    bz2.compress(b"x", 0)
except ValueError:
    _raised = True
assert _raised, "compress_level_zero_raises: expected ValueError"
print("compress_level_zero_raises OK")
"###);
    assert_output(&out, r###"compress_level_zero_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/bz2/decompress_after_eof_raises.py`.
#[test]
fn test_gen_errors_std_libs_bz2_decompress_after_eof_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bz2"
# dimension = "errors"
# case = "decompress_after_eof_raises"
# subject = "bz2.BZ2Decompressor"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bz2.py"
# status = "filled"
# ///
"""bz2.BZ2Decompressor: decompress_after_eof_raises (errors)."""
import bz2

_raised = False
try:
    (lambda d: (d.decompress(bz2.compress(b"x")), d.decompress(b"more"))) (bz2.BZ2Decompressor())
except EOFError:
    _raised = True
assert _raised, "decompress_after_eof_raises: expected EOFError"
print("decompress_after_eof_raises OK")
"###);
    assert_output(&out, r###"decompress_after_eof_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/bz2/decompress_bad_stream_raises.py`.
#[test]
fn test_gen_errors_std_libs_bz2_decompress_bad_stream_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bz2"
# dimension = "errors"
# case = "decompress_bad_stream_raises"
# subject = "bz2.decompress"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bz2.py"
# status = "filled"
# ///
"""bz2.decompress: decompress_bad_stream_raises (errors)."""
import bz2

_raised = False
try:
    bz2.decompress(b"not a bz2 stream")
except OSError:
    _raised = True
assert _raised, "decompress_bad_stream_raises: expected OSError"
print("decompress_bad_stream_raises OK")
"###);
    assert_output(&out, r###"decompress_bad_stream_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/bz2/decompressor_ctor_positional_raises.py`.
#[test]
fn test_gen_errors_std_libs_bz2_decompressor_ctor_positional_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bz2"
# dimension = "errors"
# case = "decompressor_ctor_positional_raises"
# subject = "bz2.BZ2Decompressor"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bz2.py"
# status = "filled"
# ///
"""bz2.BZ2Decompressor: decompressor_ctor_positional_raises (errors)."""
import bz2

_raised = False
try:
    bz2.BZ2Decompressor(42)
except TypeError:
    _raised = True
assert _raised, "decompressor_ctor_positional_raises: expected TypeError"
print("decompressor_ctor_positional_raises OK")
"###);
    assert_output(&out, r###"decompressor_ctor_positional_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/bz2/open_text_binary_mode_combo_raises.py`.
#[test]
fn test_gen_errors_std_libs_bz2_open_text_binary_mode_combo_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bz2"
# dimension = "errors"
# case = "open_text_binary_mode_combo_raises"
# subject = "bz2.open"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bz2.py"
# status = "filled"
# ///
"""bz2.open: open_text_binary_mode_combo_raises (errors)."""
import bz2, io

_raised = False
try:
    bz2.open(io.BytesIO(), "wbt")
except ValueError:
    _raised = True
assert _raised, "open_text_binary_mode_combo_raises: expected ValueError"
print("open_text_binary_mode_combo_raises OK")
"###);
    assert_output(&out, r###"open_text_binary_mode_combo_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/bz2/open_text_param_in_binary_mode_raises.py`.
#[test]
fn test_gen_errors_std_libs_bz2_open_text_param_in_binary_mode_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bz2"
# dimension = "errors"
# case = "open_text_param_in_binary_mode_raises"
# subject = "bz2.open"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bz2.py"
# status = "filled"
# ///
"""bz2.open: open_text_param_in_binary_mode_raises (errors)."""
import bz2, io

_raised = False
try:
    bz2.open(io.BytesIO(), "rb", encoding="utf-8")
except ValueError:
    _raised = True
assert _raised, "open_text_param_in_binary_mode_raises: expected ValueError"
print("open_text_param_in_binary_mode_raises OK")
"###);
    assert_output(&out, r###"open_text_param_in_binary_mode_raises OK
"###);
}
