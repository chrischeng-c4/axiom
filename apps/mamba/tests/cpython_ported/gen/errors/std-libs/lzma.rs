use super::super::super::super::harness::*;

/// Ported from `tests/cpython/errors/std-libs/lzma/compress_preset_out_of_range_raises.py`.
#[test]
fn test_gen_errors_std_libs_lzma_compress_preset_out_of_range_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lzma"
# dimension = "errors"
# case = "compress_preset_out_of_range_raises"
# subject = "lzma.compress"
# kind = "mechanical"
# xfail = "lzma.compress ignores the preset kwarg; preset=10 does not raise (src/runtime/stdlib/lzma_mod.rs:153)"
# mem_carveout = ""
# source = "Lib/test/test_lzma.py"
# status = "filled"
# ///
"""lzma.compress: compress_preset_out_of_range_raises (errors)."""
import lzma

_raised = False
try:
    lzma.compress(b'x', preset=10)
except lzma.LZMAError:
    _raised = True
assert _raised, "compress_preset_out_of_range_raises: expected lzma.LZMAError"
print("compress_preset_out_of_range_raises OK")
"###);
    assert_output(&out, r###"compress_preset_out_of_range_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/lzma/decompress_bad_input_raises.py`.
#[test]
fn test_gen_errors_std_libs_lzma_decompress_bad_input_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lzma"
# dimension = "errors"
# case = "decompress_bad_input_raises"
# subject = "lzma.decompress"
# kind = "mechanical"
# xfail = "lzma.decompress returns empty bytes on bad input instead of raising LZMAError (src/runtime/stdlib/lzma_mod.rs:179-188)"
# mem_carveout = ""
# source = "Lib/test/test_lzma.py"
# status = "filled"
# ///
"""lzma.decompress: decompress_bad_input_raises (errors)."""
import lzma

_raised = False
try:
    lzma.decompress(b'not lzma data')
except lzma.LZMAError:
    _raised = True
assert _raised, "decompress_bad_input_raises: expected lzma.LZMAError"
print("decompress_bad_input_raises OK")
"###);
    assert_output(&out, r###"decompress_bad_input_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/lzma/decompress_list_payload_raises.py`.
#[test]
fn test_gen_errors_std_libs_lzma_decompress_list_payload_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lzma"
# dimension = "errors"
# case = "decompress_list_payload_raises"
# subject = "lzma.decompress"
# kind = "mechanical"
# xfail = "lzma.decompress accepts a non-bytes payload (returns empty) instead of raising TypeError (src/runtime/stdlib/lzma_mod.rs:132-144)"
# mem_carveout = ""
# source = "Lib/test/test_lzma.py"
# status = "filled"
# ///
"""lzma.decompress: decompress_list_payload_raises (errors)."""
import lzma

_raised = False
try:
    lzma.decompress([])
except TypeError:
    _raised = True
assert _raised, "decompress_list_payload_raises: expected TypeError"
print("decompress_list_payload_raises OK")
"###);
    assert_output(&out, r###"decompress_list_payload_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/lzma/decompress_truncated_stream_raises.py`.
#[test]
fn test_gen_errors_std_libs_lzma_decompress_truncated_stream_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lzma"
# dimension = "errors"
# case = "decompress_truncated_stream_raises"
# subject = "lzma.decompress"
# kind = "mechanical"
# xfail = "lzma.decompress returns empty bytes on a truncated stream instead of raising (src/runtime/stdlib/lzma_mod.rs:179-188)"
# mem_carveout = ""
# source = "Lib/test/test_lzma.py"
# status = "filled"
# ///
"""lzma.decompress: decompress_truncated_stream_raises (errors)."""
import lzma

_raised = False
try:
    lzma.decompress(lzma.compress(b'hello lzma')[:4])
except lzma.LZMAError:
    _raised = True
assert _raised, "decompress_truncated_stream_raises: expected lzma.LZMAError"
print("decompress_truncated_stream_raises OK")
"###);
    assert_output(&out, r###"decompress_truncated_stream_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/lzma/encode_filter_properties_bad_type_raises.py`.
#[test]
fn test_gen_errors_std_libs_lzma_encode_filter_properties_bad_type_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lzma"
# dimension = "errors"
# case = "encode_filter_properties_bad_type_raises"
# subject = "lzma._encode_filter_properties"
# kind = "mechanical"
# xfail = "lzma._encode_filter_properties is not implemented (src/runtime/stdlib/lzma_mod.rs)"
# mem_carveout = ""
# source = "Lib/test/test_lzma.py"
# status = "filled"
# ///
"""lzma._encode_filter_properties: encode_filter_properties_bad_type_raises (errors)."""
import lzma

_raised = False
try:
    lzma._encode_filter_properties(b'not a dict')
except TypeError:
    _raised = True
assert _raised, "encode_filter_properties_bad_type_raises: expected TypeError"
print("encode_filter_properties_bad_type_raises OK")
"###);
    assert_output(&out, r###"encode_filter_properties_bad_type_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/lzma/lzmafile_bad_mode_raises.py`.
#[test]
fn test_gen_errors_std_libs_lzma_lzmafile_bad_mode_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lzma"
# dimension = "errors"
# case = "lzmafile_bad_mode_raises"
# subject = "lzma.LZMAFile"
# kind = "mechanical"
# xfail = "lzma.LZMAFile is a sentinel-string stub; constructing it does not raise (src/runtime/stdlib/lzma_mod.rs:79-80)"
# mem_carveout = ""
# source = "Lib/test/test_lzma.py"
# status = "filled"
# ///
"""lzma.LZMAFile: lzmafile_bad_mode_raises (errors)."""
import lzma

_raised = False
try:
    lzma.LZMAFile(__import__('io').BytesIO(lzma.compress(b'x')), 'rt')
except ValueError:
    _raised = True
assert _raised, "lzmafile_bad_mode_raises: expected ValueError"
print("lzmafile_bad_mode_raises OK")
"###);
    assert_output(&out, r###"lzmafile_bad_mode_raises OK
"###);
}
