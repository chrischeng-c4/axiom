use super::super::super::super::harness::*;

/// Ported from `tests/cpython/errors/std-libs/codecs/bytes_decode_nontext_codec_raises.py`.
#[test]
fn test_gen_errors_std_libs_codecs_bytes_decode_nontext_codec_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "errors"
# case = "bytes_decode_nontext_codec_raises"
# subject = "bytes.decode"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_codecs.py"
# status = "filled"
# ///
"""bytes.decode: bytes_decode_nontext_codec_raises (errors)."""
import codecs

_raised = False
try:
    b'hello'.decode('quopri_codec')
except LookupError:
    _raised = True
assert _raised, "bytes_decode_nontext_codec_raises: expected LookupError"
print("bytes_decode_nontext_codec_raises OK")
"###);
    assert_output(&out, r###"bytes_decode_nontext_codec_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/codecs/charmap_decode_dict_value_overflow_raises.py`.
#[test]
fn test_gen_errors_std_libs_codecs_charmap_decode_dict_value_overflow_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "errors"
# case = "charmap_decode_dict_value_overflow_raises"
# subject = "codecs.charmap_decode"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_codecs.py"
# status = "filled"
# ///
"""codecs.charmap_decode: charmap_decode_dict_value_overflow_raises (errors)."""
import codecs

_raised = False
try:
    codecs.charmap_decode(b'\x00', 'strict', {0: __import__('sys').maxunicode + 1})
except TypeError:
    _raised = True
assert _raised, "charmap_decode_dict_value_overflow_raises: expected TypeError"
print("charmap_decode_dict_value_overflow_raises OK")
"###);
    assert_output(&out, r###"charmap_decode_dict_value_overflow_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/codecs/charmap_decode_short_string_map_raises.py`.
#[test]
fn test_gen_errors_std_libs_codecs_charmap_decode_short_string_map_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "errors"
# case = "charmap_decode_short_string_map_raises"
# subject = "codecs.charmap_decode"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_codecs.py"
# status = "filled"
# ///
"""codecs.charmap_decode: charmap_decode_short_string_map_raises (errors)."""
import codecs

_raised = False
try:
    codecs.charmap_decode(b'\x00\x01\x02', 'strict', 'ab')
except UnicodeDecodeError:
    _raised = True
assert _raised, "charmap_decode_short_string_map_raises: expected UnicodeDecodeError"
print("charmap_decode_short_string_map_raises OK")
"###);
    assert_output(&out, r###"charmap_decode_short_string_map_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/codecs/decode_bad_utf8_strict_raises.py`.
#[test]
fn test_gen_errors_std_libs_codecs_decode_bad_utf8_strict_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "errors"
# case = "decode_bad_utf8_strict_raises"
# subject = "codecs.decode"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_codecs.py"
# status = "filled"
# ///
"""codecs.decode: decode_bad_utf8_strict_raises (errors)."""
import codecs

_raised = False
try:
    codecs.decode(b'\xff\xfe\xfd', 'utf-8')
except UnicodeDecodeError:
    _raised = True
assert _raised, "decode_bad_utf8_strict_raises: expected UnicodeDecodeError"
print("decode_bad_utf8_strict_raises OK")
"###);
    assert_output(&out, r###"decode_bad_utf8_strict_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/codecs/encode_non_ascii_strict_raises.py`.
#[test]
fn test_gen_errors_std_libs_codecs_encode_non_ascii_strict_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "errors"
# case = "encode_non_ascii_strict_raises"
# subject = "codecs.encode"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_codecs.py"
# status = "filled"
# ///
"""codecs.encode: encode_non_ascii_strict_raises (errors)."""
import codecs

_raised = False
try:
    codecs.encode('\u2603', 'ascii')
except UnicodeEncodeError:
    _raised = True
assert _raised, "encode_non_ascii_strict_raises: expected UnicodeEncodeError"
print("encode_non_ascii_strict_raises OK")
"###);
    assert_output(&out, r###"encode_non_ascii_strict_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/codecs/encode_unknown_handler_raises.py`.
#[test]
fn test_gen_errors_std_libs_codecs_encode_unknown_handler_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "errors"
# case = "encode_unknown_handler_raises"
# subject = "codecs.encode"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_codecs.py"
# status = "filled"
# ///
"""codecs.encode: encode_unknown_handler_raises (errors)."""
import codecs

_raised = False
try:
    codecs.encode('\u2603', 'ascii', 'no_such_handler')
except LookupError:
    _raised = True
assert _raised, "encode_unknown_handler_raises: expected LookupError"
print("encode_unknown_handler_raises OK")
"###);
    assert_output(&out, r###"encode_unknown_handler_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/codecs/escape_decode_truncated_raises.py`.
#[test]
fn test_gen_errors_std_libs_codecs_escape_decode_truncated_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "errors"
# case = "escape_decode_truncated_raises"
# subject = "codecs.escape_decode"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_codecs.py"
# status = "filled"
# ///
"""codecs.escape_decode: escape_decode_truncated_raises (errors)."""
import codecs

_raised = False
try:
    codecs.escape_decode(b'\\x')
except ValueError:
    _raised = True
assert _raised, "escape_decode_truncated_raises: expected ValueError"
print("escape_decode_truncated_raises OK")
"###);
    assert_output(&out, r###"escape_decode_truncated_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/codecs/getencoder_unknown_codec_raises.py`.
#[test]
fn test_gen_errors_std_libs_codecs_getencoder_unknown_codec_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "errors"
# case = "getencoder_unknown_codec_raises"
# subject = "codecs.getencoder"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_codecs.py"
# status = "filled"
# ///
"""codecs.getencoder: getencoder_unknown_codec_raises (errors)."""
import codecs

_raised = False
try:
    codecs.getencoder('__no_such_codec__')
except LookupError:
    _raised = True
assert _raised, "getencoder_unknown_codec_raises: expected LookupError"
print("getencoder_unknown_codec_raises OK")
"###);
    assert_output(&out, r###"getencoder_unknown_codec_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/codecs/lookup_unknown_codec_raises.py`.
#[test]
fn test_gen_errors_std_libs_codecs_lookup_unknown_codec_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "errors"
# case = "lookup_unknown_codec_raises"
# subject = "codecs.lookup"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_codecs.py"
# status = "filled"
# ///
"""codecs.lookup: lookup_unknown_codec_raises (errors)."""
import codecs

_raised = False
try:
    codecs.lookup('not_a_real_codec_xyz')
except LookupError:
    _raised = True
assert _raised, "lookup_unknown_codec_raises: expected LookupError"
print("lookup_unknown_codec_raises OK")
"###);
    assert_output(&out, r###"lookup_unknown_codec_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/codecs/low_level_decoder_rejects_str_raises.py`.
#[test]
fn test_gen_errors_std_libs_codecs_low_level_decoder_rejects_str_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "errors"
# case = "low_level_decoder_rejects_str_raises"
# subject = "codecs.utf_8_decode"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_codecs.py"
# status = "filled"
# ///
"""codecs.utf_8_decode: low_level_decoder_rejects_str_raises (errors)."""
import codecs

_raised = False
try:
    codecs.utf_8_decode('xxx')
except TypeError:
    _raised = True
assert _raised, "low_level_decoder_rejects_str_raises: expected TypeError"
print("low_level_decoder_rejects_str_raises OK")
"###);
    assert_output(&out, r###"low_level_decoder_rejects_str_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/codecs/str_encode_nontext_codec_raises.py`.
#[test]
fn test_gen_errors_std_libs_codecs_str_encode_nontext_codec_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "errors"
# case = "str_encode_nontext_codec_raises"
# subject = "str.encode"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_codecs.py"
# status = "filled"
# ///
"""str.encode: str_encode_nontext_codec_raises (errors)."""
import codecs

_raised = False
try:
    'msg'.encode('rot_13')
except LookupError:
    _raised = True
assert _raised, "str_encode_nontext_codec_raises: expected LookupError"
print("str_encode_nontext_codec_raises OK")
"###);
    assert_output(&out, r###"str_encode_nontext_codec_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/codecs/surrogate_strict_encode_raises.py`.
#[test]
fn test_gen_errors_std_libs_codecs_surrogate_strict_encode_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "errors"
# case = "surrogate_strict_encode_raises"
# subject = "str.encode"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_codecs.py"
# status = "filled"
# ///
"""str.encode: surrogate_strict_encode_raises (errors)."""
import codecs

_raised = False
try:
    '\ud901'.encode('utf-8')
except UnicodeEncodeError:
    _raised = True
assert _raised, "surrogate_strict_encode_raises: expected UnicodeEncodeError"
print("surrogate_strict_encode_raises OK")
"###);
    assert_output(&out, r###"surrogate_strict_encode_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/codecs/unicode_escape_decode_truncated_raises.py`.
#[test]
fn test_gen_errors_std_libs_codecs_unicode_escape_decode_truncated_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "errors"
# case = "unicode_escape_decode_truncated_raises"
# subject = "codecs.unicode_escape_decode"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_codecs.py"
# status = "filled"
# ///
"""codecs.unicode_escape_decode: unicode_escape_decode_truncated_raises (errors)."""
import codecs

_raised = False
try:
    codecs.unicode_escape_decode(b'\\x0')
except UnicodeDecodeError:
    _raised = True
assert _raised, "unicode_escape_decode_truncated_raises: expected UnicodeDecodeError"
print("unicode_escape_decode_truncated_raises OK")
"###);
    assert_output(&out, r###"unicode_escape_decode_truncated_raises OK
"###);
}
