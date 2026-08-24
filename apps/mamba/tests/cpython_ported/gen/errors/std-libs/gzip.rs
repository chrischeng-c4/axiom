use super::super::super::super::harness::*;

/// Ported from `tests/cpython/errors/std-libs/gzip/compress_level_out_of_range_raises_zliberror.py`.
#[test]
fn test_gen_errors_std_libs_gzip_compress_level_out_of_range_raises_zliberror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gzip"
# dimension = "errors"
# case = "compress_level_out_of_range_raises_zliberror"
# subject = "gzip.compress"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_gzip.py"
# status = "filled"
# ///
"""gzip.compress: compress_level_out_of_range_raises_zliberror (errors)."""
import gzip
import zlib

_raised = False
try:
    gzip.compress(b'x', compresslevel=10)
except zlib.error:
    _raised = True
assert _raised, "compress_level_out_of_range_raises_zliberror: expected zlib.error"
print("compress_level_out_of_range_raises_zliberror OK")
"###);
    assert_output(&out, r###"compress_level_out_of_range_raises_zliberror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/gzip/decompress_magic_only_raises_eoferror.py`.
#[test]
fn test_gen_errors_std_libs_gzip_decompress_magic_only_raises_eoferror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gzip"
# dimension = "errors"
# case = "decompress_magic_only_raises_eoferror"
# subject = "gzip.decompress"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_gzip.py"
# status = "filled"
# ///
"""gzip.decompress: decompress_magic_only_raises_eoferror (errors)."""
import gzip

_raised = False
try:
    gzip.decompress(gzip.compress(b'hello gzip')[:4])
except EOFError:
    _raised = True
assert _raised, "decompress_magic_only_raises_eoferror: expected EOFError"
print("decompress_magic_only_raises_eoferror OK")
"###);
    assert_output(&out, r###"decompress_magic_only_raises_eoferror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/gzip/decompress_missing_trailer_raises_eoferror.py`.
#[test]
fn test_gen_errors_std_libs_gzip_decompress_missing_trailer_raises_eoferror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gzip"
# dimension = "errors"
# case = "decompress_missing_trailer_raises_eoferror"
# subject = "gzip.decompress"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_gzip.py"
# status = "filled"
# ///
"""gzip.decompress: decompress_missing_trailer_raises_eoferror (errors)."""
import gzip

_raised = False
try:
    gzip.decompress(gzip.compress(b'trailer matters')[:-8])
except EOFError:
    _raised = True
assert _raised, "decompress_missing_trailer_raises_eoferror: expected EOFError"
print("decompress_missing_trailer_raises_eoferror OK")
"###);
    assert_output(&out, r###"decompress_missing_trailer_raises_eoferror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/gzip/decompress_non_gzip_raises_badgzipfile.py`.
#[test]
fn test_gen_errors_std_libs_gzip_decompress_non_gzip_raises_badgzipfile() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gzip"
# dimension = "errors"
# case = "decompress_non_gzip_raises_badgzipfile"
# subject = "gzip.decompress"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_gzip.py"
# status = "filled"
# ///
"""gzip.decompress: decompress_non_gzip_raises_badgzipfile (errors)."""
import gzip

_raised = False
try:
    gzip.decompress(b'not a gzip stream')
except gzip.BadGzipFile:
    _raised = True
assert _raised, "decompress_non_gzip_raises_badgzipfile: expected gzip.BadGzipFile"
print("decompress_non_gzip_raises_badgzipfile OK")
"###);
    assert_output(&out, r###"decompress_non_gzip_raises_badgzipfile OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/gzip/decompress_truncated_trailer_raises_eoferror.py`.
#[test]
fn test_gen_errors_std_libs_gzip_decompress_truncated_trailer_raises_eoferror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gzip"
# dimension = "errors"
# case = "decompress_truncated_trailer_raises_eoferror"
# subject = "gzip.decompress"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_gzip.py"
# status = "filled"
# ///
"""gzip.decompress: decompress_truncated_trailer_raises_eoferror (errors)."""
import gzip

_raised = False
try:
    gzip.decompress(gzip.compress(b'trailer matters')[:-4])
except EOFError:
    _raised = True
assert _raised, "decompress_truncated_trailer_raises_eoferror: expected EOFError"
print("decompress_truncated_trailer_raises_eoferror OK")
"###);
    assert_output(&out, r###"decompress_truncated_trailer_raises_eoferror OK
"###);
}
