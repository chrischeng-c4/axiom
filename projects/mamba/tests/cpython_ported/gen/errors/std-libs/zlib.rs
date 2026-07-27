use super::super::super::super::harness::*;

/// Ported from `tests/cpython/errors/std-libs/zlib/adler32_non_bytes_raises_typeerror.py`.
#[test]
fn test_gen_errors_std_libs_zlib_adler32_non_bytes_raises_typeerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "errors"
# case = "adler32_non_bytes_raises_typeerror"
# subject = "zlib.adler32"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""zlib.adler32: adler32_non_bytes_raises_typeerror (errors)."""
import zlib

_raised = False
try:
    zlib.adler32(42)
except TypeError:
    _raised = True
assert _raised, "adler32_non_bytes_raises_typeerror: expected TypeError"
print("adler32_non_bytes_raises_typeerror OK")
"###);
    assert_output(&out, r###"adler32_non_bytes_raises_typeerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/zlib/compress_level_out_of_range_raises_error.py`.
#[test]
fn test_gen_errors_std_libs_zlib_compress_level_out_of_range_raises_error() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "errors"
# case = "compress_level_out_of_range_raises_error"
# subject = "zlib.compress"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""zlib.compress: compress_level_out_of_range_raises_error (errors)."""
import zlib

_raised = False
try:
    zlib.compress(b'x', 10)
except zlib.error:
    _raised = True
assert _raised, "compress_level_out_of_range_raises_error: expected zlib.error"
print("compress_level_out_of_range_raises_error OK")
"###);
    assert_output(&out, r###"compress_level_out_of_range_raises_error OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/zlib/compressobj_wbits_zero_raises_valueerror.py`.
#[test]
fn test_gen_errors_std_libs_zlib_compressobj_wbits_zero_raises_valueerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "errors"
# case = "compressobj_wbits_zero_raises_valueerror"
# subject = "zlib.compressobj"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""zlib.compressobj: compressobj_wbits_zero_raises_valueerror (errors)."""
import zlib

_raised = False
try:
    zlib.compressobj(1, zlib.DEFLATED, 0)
except ValueError:
    _raised = True
assert _raised, "compressobj_wbits_zero_raises_valueerror: expected ValueError"
print("compressobj_wbits_zero_raises_valueerror OK")
"###);
    assert_output(&out, r###"compressobj_wbits_zero_raises_valueerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/zlib/crc32_non_bytes_raises_typeerror.py`.
#[test]
fn test_gen_errors_std_libs_zlib_crc32_non_bytes_raises_typeerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "errors"
# case = "crc32_non_bytes_raises_typeerror"
# subject = "zlib.crc32"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""zlib.crc32: crc32_non_bytes_raises_typeerror (errors)."""
import zlib

_raised = False
try:
    zlib.crc32('not bytes')
except TypeError:
    _raised = True
assert _raised, "crc32_non_bytes_raises_typeerror: expected TypeError"
print("crc32_non_bytes_raises_typeerror OK")
"###);
    assert_output(&out, r###"crc32_non_bytes_raises_typeerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/zlib/decompress_invalid_data_raises_error.py`.
#[test]
fn test_gen_errors_std_libs_zlib_decompress_invalid_data_raises_error() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "errors"
# case = "decompress_invalid_data_raises_error"
# subject = "zlib.decompress"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""zlib.decompress: decompress_invalid_data_raises_error (errors)."""
import zlib

_raised = False
try:
    zlib.decompress(b'not zlib compressed data xyz')
except zlib.error:
    _raised = True
assert _raised, "decompress_invalid_data_raises_error: expected zlib.error"
print("decompress_invalid_data_raises_error OK")
"###);
    assert_output(&out, r###"decompress_invalid_data_raises_error OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/zlib/decompress_truncated_stream_raises_error.py`.
#[test]
fn test_gen_errors_std_libs_zlib_decompress_truncated_stream_raises_error() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "errors"
# case = "decompress_truncated_stream_raises_error"
# subject = "zlib.decompress"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""zlib.decompress: decompress_truncated_stream_raises_error (errors)."""
import zlib

_raised = False
try:
    zlib.decompress(zlib.compress(b'hello zlib')[:3])
except zlib.error:
    _raised = True
assert _raised, "decompress_truncated_stream_raises_error: expected zlib.error"
print("decompress_truncated_stream_raises_error OK")
"###);
    assert_output(&out, r###"decompress_truncated_stream_raises_error OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/zlib/decompressobj_bad_wbits_raises_valueerror.py`.
#[test]
fn test_gen_errors_std_libs_zlib_decompressobj_bad_wbits_raises_valueerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "errors"
# case = "decompressobj_bad_wbits_raises_valueerror"
# subject = "zlib.decompressobj"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""zlib.decompressobj: decompressobj_bad_wbits_raises_valueerror (errors)."""
import zlib

_raised = False
try:
    zlib.decompressobj(99)
except ValueError:
    _raised = True
assert _raised, "decompressobj_bad_wbits_raises_valueerror: expected ValueError"
print("decompressobj_bad_wbits_raises_valueerror OK")
"###);
    assert_output(&out, r###"decompressobj_bad_wbits_raises_valueerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/zlib/decompressobj_flush_zero_length_raises_valueerror.py`.
#[test]
fn test_gen_errors_std_libs_zlib_decompressobj_flush_zero_length_raises_valueerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "errors"
# case = "decompressobj_flush_zero_length_raises_valueerror"
# subject = "zlib.decompressobj"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""zlib.decompressobj: decompressobj_flush_zero_length_raises_valueerror (errors)."""
import zlib

_raised = False
try:
    zlib.decompressobj().flush(0)
except ValueError:
    _raised = True
assert _raised, "decompressobj_flush_zero_length_raises_valueerror: expected ValueError"
print("decompressobj_flush_zero_length_raises_valueerror OK")
"###);
    assert_output(&out, r###"decompressobj_flush_zero_length_raises_valueerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/zlib/decompressobj_neg_wbits_raises_valueerror.py`.
#[test]
fn test_gen_errors_std_libs_zlib_decompressobj_neg_wbits_raises_valueerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "errors"
# case = "decompressobj_neg_wbits_raises_valueerror"
# subject = "zlib.decompressobj"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""zlib.decompressobj: decompressobj_neg_wbits_raises_valueerror (errors)."""
import zlib

_raised = False
try:
    zlib.decompressobj(-1)
except ValueError:
    _raised = True
assert _raised, "decompressobj_neg_wbits_raises_valueerror: expected ValueError"
print("decompressobj_neg_wbits_raises_valueerror OK")
"###);
    assert_output(&out, r###"decompressobj_neg_wbits_raises_valueerror OK
"###);
}
