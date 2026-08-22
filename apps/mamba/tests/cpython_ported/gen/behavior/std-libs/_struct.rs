use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/struct/bool_code_truthiness.py`.
#[test]
fn test_gen_behavior_std_libs_struct_bool_code_truthiness() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct"
# dimension = "behavior"
# case = "bool_code_truthiness"
# subject = "struct.pack"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""struct.pack: '?' packs any object by truthiness to 0x00/0x01 (0/[] -> 0, 5/[1] -> 1) and unpacks any non-zero byte to True, only b'\\x00' to False"""
import struct

# '?' packs any truthy/falsy value to a single 0x00/0x01 byte.
assert struct.pack(">?", 0) == b"\x00", "falsy -> 0"
assert struct.pack(">?", 5) == b"\x01", "truthy int -> 1"
assert struct.pack(">?", []) == b"\x00", "empty list is falsy"
assert struct.pack(">?", [1]) == b"\x01", "non-empty list is truthy"
# Any non-zero byte unpacks back to True; only 0x00 is False.
for raw in (b"\x01", b"\x7f", b"\xff", b"\xf0"):
    assert struct.unpack(">?", raw)[0] is True, f"nonzero byte -> True ({raw!r})"
assert struct.unpack(">?", b"\x00")[0] is False, "zero byte -> False"

print("bool_code_truthiness OK")
"###);
    assert_output(&out, r###"bool_code_truthiness OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/struct/double_roundtrip.py`.
#[test]
fn test_gen_behavior_std_libs_struct_double_roundtrip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct"
# dimension = "behavior"
# case = "double_roundtrip"
# subject = "struct.pack"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""struct.pack: '>d' is 64-bit IEEE 754: packing then unpacking 3.141592653589793 round-trips exactly"""
import struct

# '>d' is 8 bytes and round-trips a Python float (itself a C double) exactly.
_dv = struct.pack(">d", 3.141592653589793)
assert len(_dv) == 8, f"float64 width = {len(_dv)!r}"
_du = struct.unpack(">d", _dv)
assert _du[0] == 3.141592653589793, f"double round-trip = {_du[0]!r}"

print("double_roundtrip OK")
"###);
    assert_output(&out, r###"double_roundtrip OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/struct/endianness_byte_order.py`.
#[test]
fn test_gen_behavior_std_libs_struct_endianness_byte_order() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct"
# dimension = "behavior"
# case = "endianness_byte_order"
# subject = "struct.pack"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""struct.pack: big-endian '>I' and little-endian '<I' lay an asymmetric u32 down in opposite byte order; >i 1 == b'\\x00\\x00\\x00\\x01', <i 1 == b'\\x01\\x00\\x00\\x00'"""
import struct

# An asymmetric u32 packs in opposite byte order under the two endian prefixes.
assert struct.pack(">I", 0x01020304) == b"\x01\x02\x03\x04", "big-endian order"
assert struct.pack("<I", 0x01020304) == b"\x04\x03\x02\x01", "little-endian order"

# Big-endian writes the MSB first; little-endian the LSB first.
assert struct.pack(">i", 1) == b"\x00\x00\x00\x01", "big-endian 1"
assert struct.pack("<i", 1) == b"\x01\x00\x00\x00", "little-endian 1"

print("endianness_byte_order OK")
"###);
    assert_output(&out, r###"endianness_byte_order OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/struct/float_roundtrip.py`.
#[test]
fn test_gen_behavior_std_libs_struct_float_roundtrip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct"
# dimension = "behavior"
# case = "float_roundtrip"
# subject = "struct.pack"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""struct.pack: '>f' is 32-bit IEEE 754: packing then unpacking 3.14 round-trips to within float32 precision (abs error < 1e-3)"""
import struct

# '>f' is 4 bytes; the round-trip is only approximate at float32 precision.
_fv = struct.pack(">f", 3.14)
assert len(_fv) == 4, f"float32 width = {len(_fv)!r}"
_fu = struct.unpack(">f", _fv)
assert abs(_fu[0] - 3.14) < 0.001, f"float round-trip = {_fu[0]!r}"

print("float_roundtrip OK")
"###);
    assert_output(&out, r###"float_roundtrip OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/struct/format_char_sizes.py`.
#[test]
fn test_gen_behavior_std_libs_struct_format_char_sizes() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct"
# dimension = "behavior"
# case = "format_char_sizes"
# subject = "struct.calcsize"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""struct.calcsize: calcsize maps each standard code to its fixed width: b=1, h=2, i=4, q=8, e=2, f=4, d=8; width is independent of the byte-order prefix ('>I' and '<I' both 4)"""
import struct

# Each standard code has a fixed width.
for code, width in [("b", 1), ("h", 2), ("i", 4), ("q", 8),
                    ("e", 2), ("f", 4), ("d", 8)]:
    assert struct.calcsize(code) == width, f"calcsize({code!r}) = {struct.calcsize(code)!r}"

# Width is independent of the byte-order prefix.
assert struct.calcsize(">I") == struct.calcsize("<I") == 4, "endianness does not change width"

print("format_char_sizes OK")
"###);
    assert_output(&out, r###"format_char_sizes OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/struct/half_float_e_code.py`.
#[test]
fn test_gen_behavior_std_libs_struct_half_float_e_code() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct"
# dimension = "behavior"
# case = "half_float_e_code"
# subject = "struct.pack"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""struct.pack: 'e' is 16-bit IEEE 754: exact LE/BE bit patterns for representable values, smallest normal/subnormal round-trip, NaN packs with exponent bits set, values past the half range raise OverflowError, and nearest-even rounding at a tie"""
import math
import struct

# 'e' is two bytes wide.
assert struct.calcsize("e") == 2, "e is 16-bit"

# Exact little-endian / big-endian bit patterns for representable values.
cases = [
    (b"\x00<", 1.0),
    (b"\x00\xc0", -2.0),
    (b"\xff{", 65504.0),       # largest finite half
    (b"\x00\x00", 0.0),
    (b"\x00\x80", -0.0),
    (b"\x00|", float("inf")),
    (b"\x00\xfc", float("-inf")),
]
for le_bits, value in cases:
    be_bits = le_bits[::-1]
    assert struct.unpack("<e", le_bits)[0] == value, f"<e unpack {value!r}"
    assert struct.pack("<e", value) == le_bits, f"<e pack {value!r}"
    assert struct.unpack(">e", be_bits)[0] == value, f">e unpack {value!r}"
    assert struct.pack(">e", value) == be_bits, f">e pack {value!r}"

# Smallest normal and smallest subnormal round-trip.
assert struct.unpack("<e", b"\x00\x04")[0] == 2.0 ** -14, "smallest normal"
assert struct.unpack("<e", b"\x01\x00")[0] == 2.0 ** -24, "smallest subnormal"

# NaN bit patterns unpack to NaN, and packing a NaN sets the exponent/quiet bits.
assert math.isnan(struct.unpack("<e", b"\x00~")[0]), "NaN unpacks to nan"
packed_nan = struct.pack("<e", math.nan)
assert packed_nan[1] & 0x7e == 0x7e, "packed NaN has exponent bits set"

# Values too large for a half float overflow rather than round to inf.
for value in (65520.0, 65536.0, 1e300, -65536.0, -1e300):
    try:
        struct.pack(">e", value)
        raise AssertionError(f"expected OverflowError for {value!r}")
    except OverflowError:
        pass

# Rounding to nearest-even when a value falls between representable halves.
assert struct.pack(">e", 2.0 ** -25) == b"\x00\x00", "tie rounds down to 0"
assert struct.pack(">e", 2.0 ** -25 + 2.0 ** -35) == b"\x00\x01", "above tie rounds up"

print("half_float_e_code OK")
"###);
    assert_output(&out, r###"half_float_e_code OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/struct/iter_unpack_buffer_inputs.py`.
#[test]
fn test_gen_behavior_std_libs_struct_iter_unpack_buffer_inputs() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct"
# dimension = "behavior"
# case = "iter_unpack_buffer_inputs"
# subject = "struct.iter_unpack"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""struct.iter_unpack: Struct.iter_unpack accepts any buffer-protocol input (memoryview, bytearray) and produces the same record list as from bytes"""
import struct

data = bytes(range(1, 16))  # three ">IB" records of 5 bytes each
s = struct.Struct(">IB")
expected = [(16909060, 5), (101124105, 10), (185339150, 15)]

# Any buffer-like object works as input and yields the same records.
for view in (memoryview(data), bytearray(data)):
    records = list(s.iter_unpack(view))
    assert records == expected, f"buffer {type(view).__name__}"

print("iter_unpack_buffer_inputs OK")
"###);
    assert_output(&out, r###"iter_unpack_buffer_inputs OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/struct/iter_unpack_length_hint.py`.
#[test]
fn test_gen_behavior_std_libs_struct_iter_unpack_length_hint() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct"
# dimension = "behavior"
# case = "iter_unpack_length_hint"
# subject = "struct.iter_unpack"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""struct.iter_unpack: operator.length_hint on a struct unpack-iterator reports the count of records still to yield, decrementing to 0 as the iterator drains"""
import operator
import struct

data = bytes(range(1, 16))  # three ">IB" records of 5 bytes each
s = struct.Struct(">IB")

# length_hint reports the number of records still to yield, draining to 0.
it = s.iter_unpack(data)
assert operator.length_hint(it) == 3, "length_hint start"
next(it)
assert operator.length_hint(it) == 2, "length_hint after one"
next(it)
next(it)
assert operator.length_hint(it) == 0, "length_hint exhausted"

print("iter_unpack_length_hint OK")
"###);
    assert_output(&out, r###"iter_unpack_length_hint OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/struct/iter_unpack_type_not_constructible.py`.
#[test]
fn test_gen_behavior_std_libs_struct_iter_unpack_type_not_constructible() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct"
# dimension = "behavior"
# case = "iter_unpack_type_not_constructible"
# subject = "struct.iter_unpack"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""struct.iter_unpack: the unpack-iterator type itself cannot be instantiated directly; calling type(it)() raises TypeError"""
import struct

# The iterator type itself cannot be constructed directly.
s = struct.Struct(">IB")
iter_type = type(s.iter_unpack(b""))
try:
    iter_type()
    raise AssertionError("expected TypeError")
except TypeError:
    pass

print("iter_unpack_type_not_constructible OK")
"###);
    assert_output(&out, r###"iter_unpack_type_not_constructible OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/struct/iter_unpack_yields_records.py`.
#[test]
fn test_gen_behavior_std_libs_struct_iter_unpack_yields_records() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct"
# dimension = "behavior"
# case = "iter_unpack_yields_records"
# subject = "struct.iter_unpack"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""struct.iter_unpack: both Struct.iter_unpack and module struct.iter_unpack yield successive '>IB' records from a flat byte buffer and then raise StopIteration (stays stopped on repeat next())"""
import struct

data = bytes(range(1, 16))  # 15 bytes -> three ">IB" records of 5 bytes each

# Struct.iter_unpack yields successive records, then StopIteration.
s = struct.Struct(">IB")
it = s.iter_unpack(data)
assert next(it) == (16909060, 5), "record 0"
assert next(it) == (101124105, 10), "record 1"
assert next(it) == (185339150, 15), "record 2"
for _ in range(2):
    try:
        next(it)
        raise AssertionError("expected StopIteration")
    except StopIteration:
        pass

# The module-level struct.iter_unpack behaves the same.
it2 = struct.iter_unpack(">IB", bytes(range(1, 11)))
assert next(it2) == (16909060, 5), "module iter record 0"
assert next(it2) == (101124105, 10), "module iter record 1"
try:
    next(it2)
    raise AssertionError("expected StopIteration")
except StopIteration:
    pass

print("iter_unpack_yields_records OK")
"###);
    assert_output(&out, r###"iter_unpack_yields_records OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/struct/mixed_record_all_byteorders.py`.
#[test]
fn test_gen_behavior_std_libs_struct_mixed_record_all_byteorders() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct"
# dimension = "behavior"
# case = "mixed_record_all_byteorders"
# subject = "struct.pack"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""struct.pack: a heterogeneous 'cbHid?' record round-trips identically under every byte-order prefix ('', '@', '<', '>', '=', '!')"""
import struct

# Round-trip a heterogeneous record under every byte-order prefix.
for prefix in ("", "@", "<", ">", "=", "!"):
    fmt = prefix + "cbHid?"
    packed = struct.pack(fmt, b"a", 1, 255, 65535, 3.5, True)
    c, b, h, i, d, flag = struct.unpack(fmt, packed)
    assert c == b"a" and b == 1 and h == 255, f"int round-trip ({prefix})"
    assert i == 65535 and d == 3.5 and flag is True, f"mixed round-trip ({prefix})"

print("mixed_record_all_byteorders OK")
"###);
    assert_output(&out, r###"mixed_record_all_byteorders OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/struct/native_int_size.py`.
#[test]
fn test_gen_behavior_std_libs_struct_native_int_size() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct"
# dimension = "behavior"
# case = "native_int_size"
# subject = "struct.calcsize"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""struct.calcsize: native '@i' uses the platform int width so calcsize('@i') is 4 or 8 (and may carry native alignment), unlike the standard-size '=i'"""
import struct

# Native '@i' uses the platform int width.
_native = struct.calcsize("@i")
assert _native in (4, 8), f"native int size = {_native!r}"

# Standard-size '=i' is always exactly 4 bytes regardless of platform.
assert struct.calcsize("=i") == 4, "standard-size int is 4 bytes"

print("native_int_size OK")
"###);
    assert_output(&out, r###"native_int_size OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/struct/pack_into_writes_at_offset.py`.
#[test]
fn test_gen_behavior_std_libs_struct_pack_into_writes_at_offset() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct"
# dimension = "behavior"
# case = "pack_into_writes_at_offset"
# subject = "struct.pack_into"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""struct.pack_into: pack_into writes the packed field at the given byte offset inside a mutable buffer; two '>I' writes at offsets 0 and 4 leave the rest untouched and match the standalone pack output"""
import struct

# pack_into writes at the given offset; the two writes match standalone pack().
_buf = bytearray(8)
struct.pack_into(">I", _buf, 0, 0xDEAD)
struct.pack_into(">I", _buf, 4, 0xBEEF)
assert _buf[:4] == struct.pack(">I", 0xDEAD), "pack_into at offset 0"
assert _buf[4:] == struct.pack(">I", 0xBEEF), "pack_into at offset 4"

# Struct.pack_into writes the field at the requested offset, too.
text = b"Reykjavik rocks, eow!"
s = struct.Struct("21s")
big = bytearray(100)
s.pack_into(big, 10, text)
assert bytes(big[10:10 + len(text)]) == text, "Struct.pack_into at offset 10"

print("pack_into_writes_at_offset OK")
"###);
    assert_output(&out, r###"pack_into_writes_at_offset OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/struct/pad_and_char_codes.py`.
#[test]
fn test_gen_behavior_std_libs_struct_pad_and_char_codes() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct"
# dimension = "behavior"
# case = "pad_and_char_codes"
# subject = "struct.pack"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""struct.pack: 'c' packs one literal byte and 'x' is a zero pad byte that consumes no argument: '>xc' b'a' -> b'\\x00a', '>cx' b'a' -> b'a\\x00'"""
import struct

# 'c' packs a single literal byte.
assert struct.pack(">c", b"a") == b"a", "c code"
# 'x' is a pad byte: it contributes a zero and consumes no argument.
assert struct.pack(">xc", b"a") == b"\x00a", "x pad then c"
assert struct.pack(">cx", b"a") == b"a\x00", "c then x pad"

print("pad_and_char_codes OK")
"###);
    assert_output(&out, r###"pad_and_char_codes OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/struct/pascal_string_p_code.py`.
#[test]
fn test_gen_behavior_std_libs_struct_pascal_string_p_code() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct"
# dimension = "behavior"
# case = "pascal_string_p_code"
# subject = "struct.pack"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""struct.pack: 'Np' is a Pascal string (length byte then data): '2p' b'abc' -> b'\\x01a', '4p' b'abc' -> b'\\x03abc', the stored length caps at 255, and unpack drops the length byte"""
import struct

# 'p' is a Pascal string: the first byte is the length, then the data.
assert struct.pack("2p", b"abc") == b"\x01a", "2p: len byte + 1 char"
assert struct.pack("4p", b"abc") == b"\x03abc", "4p: len byte + 3 chars"
assert struct.unpack("4p", b"\x03abc")[0] == b"abc", "p unpack drops len byte"
# A 'p' field longer than 256 caps the stored length at 255.
big = struct.pack("1000p", b"x" * 1000)
assert big[0] == 255, "p length byte caps at 255"
assert struct.unpack("1000p", big)[0] == b"x" * 255, "p unpack truncates to 255"

print("pascal_string_p_code OK")
"###);
    assert_output(&out, r###"pascal_string_p_code OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/struct/signed_unsigned_view.py`.
#[test]
fn test_gen_behavior_std_libs_struct_signed_unsigned_view() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct"
# dimension = "behavior"
# case = "signed_unsigned_view"
# subject = "struct.unpack"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""struct.unpack: the bytes of signed '>i' -1 re-interpreted as unsigned '>I' read back as 0xFFFFFFFF (two's-complement view)"""
import struct

# Pack -1 as a signed int, then read the same bytes as unsigned.
_neg = struct.pack(">i", -1)
_unsigned = struct.unpack(">I", _neg)
assert _unsigned == (0xFFFFFFFF,), f"unsigned view of -1 = {_unsigned!r}"

print("signed_unsigned_view OK")
"###);
    assert_output(&out, r###"signed_unsigned_view OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/struct/string_field_s_code.py`.
#[test]
fn test_gen_behavior_std_libs_struct_string_field_s_code() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct"
# dimension = "behavior"
# case = "string_field_s_code"
# subject = "struct.pack"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""struct.pack: 'Ns' is a fixed N-byte field: short input zero-pads ('3s' b'ab' -> b'ab\\x00'), long input truncates ('3s' b'abcd' -> b'abc'), '0s' is empty, and unpack returns the raw N bytes"""
import struct

# 'Ns' is a fixed-width byte field: short input zero-pads, long input truncates.
assert struct.pack(">3s", b"ab") == b"ab\x00", "short 3s zero-pads"
assert struct.pack(">3s", b"abcd") == b"abc", "long 3s truncates"
# '0s' is an empty field.
assert struct.pack(">0s", b"hi") == b"", "0s is empty"
# A 5-byte field round-trips its bytes exactly.
_sv = struct.pack("5s", b"hello")
assert _sv == b"hello", f"5s pack = {_sv!r}"
assert struct.unpack("5s", _sv) == (b"hello",), "5s unpack"
# unpack returns the raw N bytes.
assert struct.unpack(">3s", b"xyz") == (b"xyz",), "3s unpack"

print("string_field_s_code OK")
"###);
    assert_output(&out, r###"string_field_s_code OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/struct/struct_caches_size_and_format.py`.
#[test]
fn test_gen_behavior_std_libs_struct_struct_caches_size_and_format() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct"
# dimension = "behavior"
# case = "struct_caches_size_and_format"
# subject = "struct.Struct"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""struct.Struct: a compiled Struct('>IB') caches .size == 5 and .format == '>IB'; a Struct built from a bytes format normalizes .format back to the same str"""
import struct

# A compiled Struct caches its size and format.
s = struct.Struct(">IB")
assert s.size == 5, f"size = {s.size!r}"
assert s.format == ">IB", f"format = {s.format!r}"

# A Struct built from a bytes format normalizes .format back to the same str.
s2 = struct.Struct(s.format.encode())
assert s2.format == s.format, "bytes format normalizes to the same string"

print("struct_caches_size_and_format OK")
"###);
    assert_output(&out, r###"struct_caches_size_and_format OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/struct/struct_reinit_rebinds_format.py`.
#[test]
fn test_gen_behavior_std_libs_struct_struct_reinit_rebinds_format() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct"
# dimension = "behavior"
# case = "struct_reinit_rebinds_format"
# subject = "struct.Struct"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""struct.Struct: calling __init__ again on a Struct rebinds it to a new format: Struct('i') then __init__('ii') changes .size 4 -> 8 and round-trips two ints"""
import struct

# Re-running __init__ rebinds the Struct to a new format.
r = struct.Struct("i")
assert r.size == 4, "single int size"
r.__init__("ii")
assert r.size == 8, "reinit to two ints"
assert r.unpack(struct.pack("ii", 7, 9)) == (7, 9), "reinit round-trip"

print("struct_reinit_rebinds_format OK")
"###);
    assert_output(&out, r###"struct_reinit_rebinds_format OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/struct/struct_test__test_1530559.py`.
#[test]
fn test_gen_behavior_std_libs_struct_struct_test__test_1530559() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct"
# dimension = "behavior"
# case = "struct_test__test_1530559"
# subject = "cpython.test_struct.StructTest.test_1530559"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_struct.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_struct.py::StructTest::test_1530559
"""Auto-ported test: StructTest::test_1530559 (CPython 3.12 oracle)."""


from collections import abc
import array
import gc
import math
import operator
import unittest
import struct
import sys
import weakref
from test import support
from test.support import import_helper
from test.support.script_helper import assert_python_ok


ISBIGENDIAN = sys.byteorder == 'big'

integer_codes = ('b', 'B', 'h', 'H', 'i', 'I', 'l', 'L', 'q', 'Q', 'n', 'N')

byteorders = ('', '@', '=', '<', '>', '!')

def iter_integer_formats(byteorders=byteorders):
    for code in integer_codes:
        for byteorder in byteorders:
            if byteorder not in ('', '@') and code in ('n', 'N'):
                continue
            yield (code, byteorder)

def string_reverse(s):
    return s[::-1]

def bigendian_to_native(value):
    if ISBIGENDIAN:
        return value
    else:
        return string_reverse(value)


# --- test body ---
for code, byteorder in iter_integer_formats():
    format = byteorder + code

    try:
        struct.pack(format, 1.0)
        raise AssertionError('expected struct.error')
    except struct.error:
        pass

    try:
        struct.pack(format, 1.5)
        raise AssertionError('expected struct.error')
    except struct.error:
        pass

try:
    struct.pack('P', 1.0)
    raise AssertionError('expected struct.error')
except struct.error:
    pass

try:
    struct.pack('P', 1.5)
    raise AssertionError('expected struct.error')
except struct.error:
    pass
print("StructTest::test_1530559: ok")
"###);
    assert_output(&out, r###"StructTest::test_1530559: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/struct/struct_test__test_bool.py`.
#[test]
fn test_gen_behavior_std_libs_struct_struct_test__test_bool() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct"
# dimension = "behavior"
# case = "struct_test__test_bool"
# subject = "cpython.test_struct.StructTest.test_bool"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_struct.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_struct.py::StructTest::test_bool
"""Auto-ported test: StructTest::test_bool (CPython 3.12 oracle)."""


from collections import abc
import array
import gc
import math
import operator
import unittest
import struct
import sys
import weakref
from test import support
from test.support import import_helper
from test.support.script_helper import assert_python_ok


ISBIGENDIAN = sys.byteorder == 'big'

integer_codes = ('b', 'B', 'h', 'H', 'i', 'I', 'l', 'L', 'q', 'Q', 'n', 'N')

byteorders = ('', '@', '=', '<', '>', '!')

def iter_integer_formats(byteorders=byteorders):
    for code in integer_codes:
        for byteorder in byteorders:
            if byteorder not in ('', '@') and code in ('n', 'N'):
                continue
            yield (code, byteorder)

def string_reverse(s):
    return s[::-1]

def bigendian_to_native(value):
    if ISBIGENDIAN:
        return value
    else:
        return string_reverse(value)


# --- test body ---
class ExplodingBool(object):

    def __bool__(self):
        raise OSError
for prefix in tuple('<>!=') + ('',):
    false = ((), [], [], '', 0)
    true = ([1], 'test', 5, -1, 4294967295 + 1, 4294967295 / 2)
    falseFormat = prefix + '?' * len(false)
    packedFalse = struct.pack(falseFormat, *false)
    unpackedFalse = struct.unpack(falseFormat, packedFalse)
    trueFormat = prefix + '?' * len(true)
    packedTrue = struct.pack(trueFormat, *true)
    unpackedTrue = struct.unpack(trueFormat, packedTrue)

    assert len(true) == len(unpackedTrue)

    assert len(false) == len(unpackedFalse)
    for t in unpackedFalse:

        assert not t
    for t in unpackedTrue:

        assert t
    packed = struct.pack(prefix + '?', 1)

    assert len(packed) == struct.calcsize(prefix + '?')
    if len(packed) != 1:

        assert not prefix
    try:
        struct.pack(prefix + '?', ExplodingBool())
    except OSError:
        pass
    else:

        raise AssertionError('Expected OSError: struct.pack(%r, ExplodingBool())' % (prefix + '?'))
for c in [b'\x01', b'\x7f', b'\xff', b'\x0f', b'\xf0']:

    assert struct.unpack('>?', c)[0]

    assert struct.unpack('<?', c)[0]

    assert struct.unpack('=?', c)[0]

    assert struct.unpack('@?', c)[0]
print("StructTest::test_bool: ok")
"###);
    assert_output(&out, r###"StructTest::test_bool: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/struct/struct_test__test_boundary_error_message.py`.
#[test]
fn test_gen_behavior_std_libs_struct_struct_test__test_boundary_error_message() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct"
# dimension = "behavior"
# case = "struct_test__test_boundary_error_message"
# subject = "cpython.test_struct.StructTest.test_boundary_error_message"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_struct.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_struct.py::StructTest::test_boundary_error_message
"""Auto-ported test: StructTest::test_boundary_error_message (CPython 3.12 oracle)."""


from collections import abc
import array
import gc
import math
import operator
import unittest
import struct
import sys
import weakref
from test import support
from test.support import import_helper
from test.support.script_helper import assert_python_ok


ISBIGENDIAN = sys.byteorder == 'big'

integer_codes = ('b', 'B', 'h', 'H', 'i', 'I', 'l', 'L', 'q', 'Q', 'n', 'N')

byteorders = ('', '@', '=', '<', '>', '!')

def iter_integer_formats(byteorders=byteorders):
    for code in integer_codes:
        for byteorder in byteorders:
            if byteorder not in ('', '@') and code in ('n', 'N'):
                continue
            yield (code, byteorder)

def string_reverse(s):
    return s[::-1]

def bigendian_to_native(value):
    if ISBIGENDIAN:
        return value
    else:
        return string_reverse(value)


# --- test body ---
regex1 = 'pack_into requires a buffer of at least 6 bytes for packing 1 bytes at offset 5 \\(actual buffer size is 1\\)'
try:
    struct.pack_into('b', bytearray(1), 5, 1)
    raise AssertionError('expected struct.error')
except struct.error as _aR_e:
    import re as _re_aR
    assert _re_aR.search(regex1, str(_aR_e))
regex2 = 'unpack_from requires a buffer of at least 6 bytes for unpacking 1 bytes at offset 5 \\(actual buffer size is 1\\)'
try:
    struct.unpack_from('b', bytearray(1), 5)
    raise AssertionError('expected struct.error')
except struct.error as _aR_e:
    import re as _re_aR
    assert _re_aR.search(regex2, str(_aR_e))
print("StructTest::test_boundary_error_message: ok")
"###);
    assert_output(&out, r###"StructTest::test_boundary_error_message: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/struct/struct_test__test_boundary_error_message_with_negative_offset.py`.
#[test]
fn test_gen_behavior_std_libs_struct_struct_test__test_boundary_error_message_with_negative_offset() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct"
# dimension = "behavior"
# case = "struct_test__test_boundary_error_message_with_negative_offset"
# subject = "cpython.test_struct.StructTest.test_boundary_error_message_with_negative_offset"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_struct.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_struct.py::StructTest::test_boundary_error_message_with_negative_offset
"""Auto-ported test: StructTest::test_boundary_error_message_with_negative_offset (CPython 3.12 oracle)."""


from collections import abc
import array
import gc
import math
import operator
import unittest
import struct
import sys
import weakref
from test import support
from test.support import import_helper
from test.support.script_helper import assert_python_ok


ISBIGENDIAN = sys.byteorder == 'big'

integer_codes = ('b', 'B', 'h', 'H', 'i', 'I', 'l', 'L', 'q', 'Q', 'n', 'N')

byteorders = ('', '@', '=', '<', '>', '!')

def iter_integer_formats(byteorders=byteorders):
    for code in integer_codes:
        for byteorder in byteorders:
            if byteorder not in ('', '@') and code in ('n', 'N'):
                continue
            yield (code, byteorder)

def string_reverse(s):
    return s[::-1]

def bigendian_to_native(value):
    if ISBIGENDIAN:
        return value
    else:
        return string_reverse(value)


# --- test body ---
byte_list = bytearray(10)
try:
    struct.pack_into('<I', byte_list, -2, 123)
    raise AssertionError('expected struct.error')
except struct.error as _aR_e:
    import re as _re_aR
    assert _re_aR.search('no space to pack 4 bytes at offset -2', str(_aR_e))
try:
    struct.pack_into('<B', byte_list, -11, 123)
    raise AssertionError('expected struct.error')
except struct.error as _aR_e:
    import re as _re_aR
    assert _re_aR.search('offset -11 out of range for 10-byte buffer', str(_aR_e))
try:
    struct.unpack_from('<I', byte_list, -2)
    raise AssertionError('expected struct.error')
except struct.error as _aR_e:
    import re as _re_aR
    assert _re_aR.search('not enough data to unpack 4 bytes at offset -2', str(_aR_e))
try:
    struct.unpack_from('<B', byte_list, -11)
    raise AssertionError('expected struct.error')
except struct.error as _aR_e:
    import re as _re_aR
    assert _re_aR.search('offset -11 out of range for 10-byte buffer', str(_aR_e))
print("StructTest::test_boundary_error_message_with_negative_offset: ok")
"###);
    assert_output(&out, r###"StructTest::test_boundary_error_message_with_negative_offset: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/struct/struct_test__test_consistence.py`.
#[test]
fn test_gen_behavior_std_libs_struct_struct_test__test_consistence() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct"
# dimension = "behavior"
# case = "struct_test__test_consistence"
# subject = "cpython.test_struct.StructTest.test_consistence"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_struct.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_struct.py::StructTest::test_consistence
"""Auto-ported test: StructTest::test_consistence (CPython 3.12 oracle)."""


from collections import abc
import array
import gc
import math
import operator
import unittest
import struct
import sys
import weakref
from test import support
from test.support import import_helper
from test.support.script_helper import assert_python_ok


ISBIGENDIAN = sys.byteorder == 'big'

integer_codes = ('b', 'B', 'h', 'H', 'i', 'I', 'l', 'L', 'q', 'Q', 'n', 'N')

byteorders = ('', '@', '=', '<', '>', '!')

def iter_integer_formats(byteorders=byteorders):
    for code in integer_codes:
        for byteorder in byteorders:
            if byteorder not in ('', '@') and code in ('n', 'N'):
                continue
            yield (code, byteorder)

def string_reverse(s):
    return s[::-1]

def bigendian_to_native(value):
    if ISBIGENDIAN:
        return value
    else:
        return string_reverse(value)


# --- test body ---

try:
    struct.calcsize('Z')
    raise AssertionError('expected struct.error')
except struct.error:
    pass
sz = struct.calcsize('i')

assert sz * 3 == struct.calcsize('iii')
fmt = 'cbxxxxxxhhhhiillffd?'
fmt3 = '3c3b18x12h6i6l6f3d3?'
sz = struct.calcsize(fmt)
sz3 = struct.calcsize(fmt3)

assert sz * 3 == sz3

try:
    struct.pack('iii', 3)
    raise AssertionError('expected struct.error')
except struct.error:
    pass

try:
    struct.pack('i', 3, 3, 3)
    raise AssertionError('expected struct.error')
except struct.error:
    pass

try:
    struct.pack('i', 'foo')
    raise AssertionError('expected (TypeError, struct.error)')
except (TypeError, struct.error):
    pass

try:
    struct.pack('P', 'foo')
    raise AssertionError('expected (TypeError, struct.error)')
except (TypeError, struct.error):
    pass

try:
    struct.unpack('d', b'flap')
    raise AssertionError('expected struct.error')
except struct.error:
    pass
s = struct.pack('ii', 1, 2)

try:
    struct.unpack('iii', s)
    raise AssertionError('expected struct.error')
except struct.error:
    pass

try:
    struct.unpack('i', s)
    raise AssertionError('expected struct.error')
except struct.error:
    pass
print("StructTest::test_consistence: ok")
"###);
    assert_output(&out, r###"StructTest::test_consistence: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/struct/struct_test__test_format_attr.py`.
#[test]
fn test_gen_behavior_std_libs_struct_struct_test__test_format_attr() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct"
# dimension = "behavior"
# case = "struct_test__test_format_attr"
# subject = "cpython.test_struct.StructTest.test_format_attr"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_struct.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_struct.py::StructTest::test_format_attr
"""Auto-ported test: StructTest::test_format_attr (CPython 3.12 oracle)."""


from collections import abc
import array
import gc
import math
import operator
import unittest
import struct
import sys
import weakref
from test import support
from test.support import import_helper
from test.support.script_helper import assert_python_ok


ISBIGENDIAN = sys.byteorder == 'big'

integer_codes = ('b', 'B', 'h', 'H', 'i', 'I', 'l', 'L', 'q', 'Q', 'n', 'N')

byteorders = ('', '@', '=', '<', '>', '!')

def iter_integer_formats(byteorders=byteorders):
    for code in integer_codes:
        for byteorder in byteorders:
            if byteorder not in ('', '@') and code in ('n', 'N'):
                continue
            yield (code, byteorder)

def string_reverse(s):
    return s[::-1]

def bigendian_to_native(value):
    if ISBIGENDIAN:
        return value
    else:
        return string_reverse(value)


# --- test body ---
s = struct.Struct('=i2H')

assert s.format == '=i2H'
s2 = struct.Struct(s.format.encode())

assert s2.format == s.format
print("StructTest::test_format_attr: ok")
"###);
    assert_output(&out, r###"StructTest::test_format_attr: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/struct/struct_test__test_isbigendian.py`.
#[test]
fn test_gen_behavior_std_libs_struct_struct_test__test_isbigendian() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct"
# dimension = "behavior"
# case = "struct_test__test_isbigendian"
# subject = "cpython.test_struct.StructTest.test_isbigendian"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_struct.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_struct.py::StructTest::test_isbigendian
"""Auto-ported test: StructTest::test_isbigendian (CPython 3.12 oracle)."""


from collections import abc
import array
import gc
import math
import operator
import unittest
import struct
import sys
import weakref
from test import support
from test.support import import_helper
from test.support.script_helper import assert_python_ok


ISBIGENDIAN = sys.byteorder == 'big'

integer_codes = ('b', 'B', 'h', 'H', 'i', 'I', 'l', 'L', 'q', 'Q', 'n', 'N')

byteorders = ('', '@', '=', '<', '>', '!')

def iter_integer_formats(byteorders=byteorders):
    for code in integer_codes:
        for byteorder in byteorders:
            if byteorder not in ('', '@') and code in ('n', 'N'):
                continue
            yield (code, byteorder)

def string_reverse(s):
    return s[::-1]

def bigendian_to_native(value):
    if ISBIGENDIAN:
        return value
    else:
        return string_reverse(value)


# --- test body ---

assert (struct.pack('=i', 1)[0] == 0) == ISBIGENDIAN
print("StructTest::test_isbigendian: ok")
"###);
    assert_output(&out, r###"StructTest::test_isbigendian: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/struct/struct_test__test_issue29802.py`.
#[test]
fn test_gen_behavior_std_libs_struct_struct_test__test_issue29802() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct"
# dimension = "behavior"
# case = "struct_test__test_issue29802"
# subject = "cpython.test_struct.StructTest.test_issue29802"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_struct.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_struct.py::StructTest::test_issue29802
"""Auto-ported test: StructTest::test_issue29802 (CPython 3.12 oracle)."""


from collections import abc
import array
import gc
import math
import operator
import unittest
import struct
import sys
import weakref
from test import support
from test.support import import_helper
from test.support.script_helper import assert_python_ok


ISBIGENDIAN = sys.byteorder == 'big'

integer_codes = ('b', 'B', 'h', 'H', 'i', 'I', 'l', 'L', 'q', 'Q', 'n', 'N')

byteorders = ('', '@', '=', '<', '>', '!')

def iter_integer_formats(byteorders=byteorders):
    for code in integer_codes:
        for byteorder in byteorders:
            if byteorder not in ('', '@') and code in ('n', 'N'):
                continue
            yield (code, byteorder)

def string_reverse(s):
    return s[::-1]

def bigendian_to_native(value):
    if ISBIGENDIAN:
        return value
    else:
        return string_reverse(value)


# --- test body ---
try:
    struct.unpack('b', 0)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

assert struct.unpack('b', b'a') == (b'a'[0],)
print("StructTest::test_issue29802: ok")
"###);
    assert_output(&out, r###"StructTest::test_issue29802: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/struct/struct_test__test_issue35714.py`.
#[test]
fn test_gen_behavior_std_libs_struct_struct_test__test_issue35714() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct"
# dimension = "behavior"
# case = "struct_test__test_issue35714"
# subject = "cpython.test_struct.StructTest.test_issue35714"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_struct.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_struct.py::StructTest::test_issue35714
"""Auto-ported test: StructTest::test_issue35714 (CPython 3.12 oracle)."""


from collections import abc
import array
import gc
import math
import operator
import unittest
import struct
import sys
import weakref
from test import support
from test.support import import_helper
from test.support.script_helper import assert_python_ok


ISBIGENDIAN = sys.byteorder == 'big'

integer_codes = ('b', 'B', 'h', 'H', 'i', 'I', 'l', 'L', 'q', 'Q', 'n', 'N')

byteorders = ('', '@', '=', '<', '>', '!')

def iter_integer_formats(byteorders=byteorders):
    for code in integer_codes:
        for byteorder in byteorders:
            if byteorder not in ('', '@') and code in ('n', 'N'):
                continue
            yield (code, byteorder)

def string_reverse(s):
    return s[::-1]

def bigendian_to_native(value):
    if ISBIGENDIAN:
        return value
    else:
        return string_reverse(value)


# --- test body ---
for s in ('\x00', '2\x00i', b'\x00'):
    try:
        struct.calcsize(s)
        raise AssertionError('expected struct.error')
    except struct.error as _aR_e:
        import re as _re_aR
        assert _re_aR.search('embedded null character', str(_aR_e))
print("StructTest::test_issue35714: ok")
"###);
    assert_output(&out, r###"StructTest::test_issue35714: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/struct/struct_test__test_p_code.py`.
#[test]
fn test_gen_behavior_std_libs_struct_struct_test__test_p_code() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct"
# dimension = "behavior"
# case = "struct_test__test_p_code"
# subject = "cpython.test_struct.StructTest.test_p_code"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_struct.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_struct.py::StructTest::test_p_code
"""Auto-ported test: StructTest::test_p_code (CPython 3.12 oracle)."""


from collections import abc
import array
import gc
import math
import operator
import unittest
import struct
import sys
import weakref
from test import support
from test.support import import_helper
from test.support.script_helper import assert_python_ok


ISBIGENDIAN = sys.byteorder == 'big'

integer_codes = ('b', 'B', 'h', 'H', 'i', 'I', 'l', 'L', 'q', 'Q', 'n', 'N')

byteorders = ('', '@', '=', '<', '>', '!')

def iter_integer_formats(byteorders=byteorders):
    for code in integer_codes:
        for byteorder in byteorders:
            if byteorder not in ('', '@') and code in ('n', 'N'):
                continue
            yield (code, byteorder)

def string_reverse(s):
    return s[::-1]

def bigendian_to_native(value):
    if ISBIGENDIAN:
        return value
    else:
        return string_reverse(value)


# --- test body ---
for code, input, expected, expectedback in [('0p', b'abc', b'', b''), ('p', b'abc', b'\x00', b''), ('1p', b'abc', b'\x00', b''), ('2p', b'abc', b'\x01a', b'a'), ('3p', b'abc', b'\x02ab', b'ab'), ('4p', b'abc', b'\x03abc', b'abc'), ('5p', b'abc', b'\x03abc\x00', b'abc'), ('6p', b'abc', b'\x03abc\x00\x00', b'abc'), ('1000p', b'x' * 1000, b'\xff' + b'x' * 999, b'x' * 255)]:
    got = struct.pack(code, input)

    assert got == expected
    got, = struct.unpack(code, got)

    assert got == expectedback
print("StructTest::test_p_code: ok")
"###);
    assert_output(&out, r###"StructTest::test_p_code: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/struct/struct_test__test_struct_reinitialization.py`.
#[test]
fn test_gen_behavior_std_libs_struct_struct_test__test_struct_reinitialization() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct"
# dimension = "behavior"
# case = "struct_test__test_struct_reinitialization"
# subject = "cpython.test_struct.StructTest.test_Struct_reinitialization"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_struct.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_struct.py::StructTest::test_Struct_reinitialization
"""Auto-ported test: StructTest::test_Struct_reinitialization (CPython 3.12 oracle)."""


from collections import abc
import array
import gc
import math
import operator
import unittest
import struct
import sys
import weakref
from test import support
from test.support import import_helper
from test.support.script_helper import assert_python_ok


ISBIGENDIAN = sys.byteorder == 'big'

integer_codes = ('b', 'B', 'h', 'H', 'i', 'I', 'l', 'L', 'q', 'Q', 'n', 'N')

byteorders = ('', '@', '=', '<', '>', '!')

def iter_integer_formats(byteorders=byteorders):
    for code in integer_codes:
        for byteorder in byteorders:
            if byteorder not in ('', '@') and code in ('n', 'N'):
                continue
            yield (code, byteorder)

def string_reverse(s):
    return s[::-1]

def bigendian_to_native(value):
    if ISBIGENDIAN:
        return value
    else:
        return string_reverse(value)


# --- test body ---
s = struct.Struct('i')
s.__init__('ii')
print("StructTest::test_Struct_reinitialization: ok")
"###);
    assert_output(&out, r###"StructTest::test_Struct_reinitialization: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/struct/struct_test__test_trailing_counter.py`.
#[test]
fn test_gen_behavior_std_libs_struct_struct_test__test_trailing_counter() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct"
# dimension = "behavior"
# case = "struct_test__test_trailing_counter"
# subject = "cpython.test_struct.StructTest.test_trailing_counter"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_struct.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_struct.py::StructTest::test_trailing_counter
"""Auto-ported test: StructTest::test_trailing_counter (CPython 3.12 oracle)."""


from collections import abc
import array
import gc
import math
import operator
import unittest
import struct
import sys
import weakref
from test import support
from test.support import import_helper
from test.support.script_helper import assert_python_ok


ISBIGENDIAN = sys.byteorder == 'big'

integer_codes = ('b', 'B', 'h', 'H', 'i', 'I', 'l', 'L', 'q', 'Q', 'n', 'N')

byteorders = ('', '@', '=', '<', '>', '!')

def iter_integer_formats(byteorders=byteorders):
    for code in integer_codes:
        for byteorder in byteorders:
            if byteorder not in ('', '@') and code in ('n', 'N'):
                continue
            yield (code, byteorder)

def string_reverse(s):
    return s[::-1]

def bigendian_to_native(value):
    if ISBIGENDIAN:
        return value
    else:
        return string_reverse(value)


# --- test body ---
store = array.array('b', b' ' * 100)

try:
    struct.pack('12345')
    raise AssertionError('expected struct.error')
except struct.error:
    pass

try:
    struct.unpack('12345', b'')
    raise AssertionError('expected struct.error')
except struct.error:
    pass

try:
    struct.pack_into('12345', store, 0)
    raise AssertionError('expected struct.error')
except struct.error:
    pass

try:
    struct.unpack_from('12345', store, 0)
    raise AssertionError('expected struct.error')
except struct.error:
    pass

try:
    struct.pack('c12345', 'x')
    raise AssertionError('expected struct.error')
except struct.error:
    pass

try:
    struct.unpack('c12345', b'x')
    raise AssertionError('expected struct.error')
except struct.error:
    pass

try:
    struct.pack_into('c12345', store, 0, 'x')
    raise AssertionError('expected struct.error')
except struct.error:
    pass

try:
    struct.unpack_from('c12345', store, 0)
    raise AssertionError('expected struct.error')
except struct.error:
    pass

try:
    struct.pack('14s42', 'spam and eggs')
    raise AssertionError('expected struct.error')
except struct.error:
    pass

try:
    struct.unpack('14s42', b'spam and eggs')
    raise AssertionError('expected struct.error')
except struct.error:
    pass

try:
    struct.pack_into('14s42', store, 0, 'spam and eggs')
    raise AssertionError('expected struct.error')
except struct.error:
    pass

try:
    struct.unpack_from('14s42', store, 0)
    raise AssertionError('expected struct.error')
except struct.error:
    pass
print("StructTest::test_trailing_counter: ok")
"###);
    assert_output(&out, r###"StructTest::test_trailing_counter: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/struct/struct_test__test_transitiveness.py`.
#[test]
fn test_gen_behavior_std_libs_struct_struct_test__test_transitiveness() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct"
# dimension = "behavior"
# case = "struct_test__test_transitiveness"
# subject = "cpython.test_struct.StructTest.test_transitiveness"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_struct.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_struct.py::StructTest::test_transitiveness
"""Auto-ported test: StructTest::test_transitiveness (CPython 3.12 oracle)."""


from collections import abc
import array
import gc
import math
import operator
import unittest
import struct
import sys
import weakref
from test import support
from test.support import import_helper
from test.support.script_helper import assert_python_ok


ISBIGENDIAN = sys.byteorder == 'big'

integer_codes = ('b', 'B', 'h', 'H', 'i', 'I', 'l', 'L', 'q', 'Q', 'n', 'N')

byteorders = ('', '@', '=', '<', '>', '!')

def iter_integer_formats(byteorders=byteorders):
    for code in integer_codes:
        for byteorder in byteorders:
            if byteorder not in ('', '@') and code in ('n', 'N'):
                continue
            yield (code, byteorder)

def string_reverse(s):
    return s[::-1]

def bigendian_to_native(value):
    if ISBIGENDIAN:
        return value
    else:
        return string_reverse(value)


# --- test body ---
c = b'a'
b = 1
h = 255
i = 65535
l = 65536
f = 3.1415
d = 3.1415
t = True
for prefix in ('', '@', '<', '>', '=', '!'):
    for format in ('xcbhilfd?', 'xcBHILfd?'):
        format = prefix + format
        s = struct.pack(format, c, b, h, i, l, f, d, t)
        cp, bp, hp, ip, lp, fp, dp, tp = struct.unpack(format, s)

        assert cp == c

        assert bp == b

        assert hp == h

        assert ip == i

        assert lp == l

        assert int(100 * fp) == int(100 * f)

        assert int(100 * dp) == int(100 * d)

        assert tp == t
print("StructTest::test_transitiveness: ok")
"###);
    assert_output(&out, r###"StructTest::test_transitiveness: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/struct/struct_test__test_unpack_with_buffer.py`.
#[test]
fn test_gen_behavior_std_libs_struct_struct_test__test_unpack_with_buffer() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct"
# dimension = "behavior"
# case = "struct_test__test_unpack_with_buffer"
# subject = "cpython.test_struct.StructTest.test_unpack_with_buffer"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_struct.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_struct.py::StructTest::test_unpack_with_buffer
"""Auto-ported test: StructTest::test_unpack_with_buffer (CPython 3.12 oracle)."""


from collections import abc
import array
import gc
import math
import operator
import unittest
import struct
import sys
import weakref
from test import support
from test.support import import_helper
from test.support.script_helper import assert_python_ok


ISBIGENDIAN = sys.byteorder == 'big'

integer_codes = ('b', 'B', 'h', 'H', 'i', 'I', 'l', 'L', 'q', 'Q', 'n', 'N')

byteorders = ('', '@', '=', '<', '>', '!')

def iter_integer_formats(byteorders=byteorders):
    for code in integer_codes:
        for byteorder in byteorders:
            if byteorder not in ('', '@') and code in ('n', 'N'):
                continue
            yield (code, byteorder)

def string_reverse(s):
    return s[::-1]

def bigendian_to_native(value):
    if ISBIGENDIAN:
        return value
    else:
        return string_reverse(value)


# --- test body ---
data1 = array.array('B', b'\x124Vx')
data2 = memoryview(b'\x124Vx')
for data in [data1, data2]:
    value, = struct.unpack('>I', data)

    assert value == 305419896
print("StructTest::test_unpack_with_buffer: ok")
"###);
    assert_output(&out, r###"StructTest::test_unpack_with_buffer: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/struct/unpack_from_buffer_protocol.py`.
#[test]
fn test_gen_behavior_std_libs_struct_unpack_from_buffer_protocol() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct"
# dimension = "behavior"
# case = "unpack_from_buffer_protocol"
# subject = "struct.unpack"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""struct.unpack: unpack reads from any buffer-protocol object (memoryview, bytearray), not just bytes: '>I' on b'\\x12\\x34\\x56\\x78' yields 0x12345678 from each"""
import struct

# unpack reads from any buffer-protocol object, not just bytes.
for buf in (memoryview(b"\x12\x34\x56\x78"), bytearray(b"\x12\x34\x56\x78")):
    (value,) = struct.unpack(">I", buf)
    assert value == 0x12345678, f"buffer unpack from {type(buf).__name__}"

print("unpack_from_buffer_protocol OK")
"###);
    assert_output(&out, r###"unpack_from_buffer_protocol OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/struct/unpack_from_slides_window.py`.
#[test]
fn test_gen_behavior_std_libs_struct_unpack_from_slides_window() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct"
# dimension = "behavior"
# case = "unpack_from_slides_window"
# subject = "struct.unpack_from"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""struct.unpack_from: unpack_from slides a fixed-size '4s' window across bytes and bytearray inputs at each offset, defaults offset to 0, and accepts keyword args buffer=/offset="""
import struct

field = struct.Struct("4s")
for cls in (bytes, bytearray):
    data = cls(b"abcd01234")
    # Defaults offset to 0, then slides the window across the buffer.
    assert field.unpack_from(data) == (b"abcd",), f"default offset ({cls.__name__})"
    assert field.unpack_from(data, 2) == (b"cd01",), f"offset 2 ({cls.__name__})"
    for i in range(6):
        assert field.unpack_from(data, i) == (bytes(data[i:i + 4]),), f"window at {i}"

# unpack_from accepts keyword arguments.
assert field.unpack_from(buffer=b"abcd01234", offset=2) == (b"cd01",), "keyword args"

print("unpack_from_slides_window OK")
"###);
    assert_output(&out, r###"unpack_from_slides_window OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/struct/unpack_iterator_test__test_arbitrary_buffer.py`.
#[test]
fn test_gen_behavior_std_libs_struct_unpack_iterator_test__test_arbitrary_buffer() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct"
# dimension = "behavior"
# case = "unpack_iterator_test__test_arbitrary_buffer"
# subject = "cpython.test_struct.UnpackIteratorTest.test_arbitrary_buffer"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_struct.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_struct.py::UnpackIteratorTest::test_arbitrary_buffer
"""Auto-ported test: UnpackIteratorTest::test_arbitrary_buffer (CPython 3.12 oracle)."""


from collections import abc
import array
import gc
import math
import operator
import unittest
import struct
import sys
import weakref
from test import support
from test.support import import_helper
from test.support.script_helper import assert_python_ok


ISBIGENDIAN = sys.byteorder == 'big'

integer_codes = ('b', 'B', 'h', 'H', 'i', 'I', 'l', 'L', 'q', 'Q', 'n', 'N')

byteorders = ('', '@', '=', '<', '>', '!')

def iter_integer_formats(byteorders=byteorders):
    for code in integer_codes:
        for byteorder in byteorders:
            if byteorder not in ('', '@') and code in ('n', 'N'):
                continue
            yield (code, byteorder)

def string_reverse(s):
    return s[::-1]

def bigendian_to_native(value):
    if ISBIGENDIAN:
        return value
    else:
        return string_reverse(value)


# --- test body ---
s = struct.Struct('>IB')
b = bytes(range(1, 11))
it = s.iter_unpack(memoryview(b))

assert next(it) == (16909060, 5)

assert next(it) == (101124105, 10)

try:
    next(it)
    raise AssertionError('expected StopIteration')
except StopIteration:
    pass

try:
    next(it)
    raise AssertionError('expected StopIteration')
except StopIteration:
    pass
print("UnpackIteratorTest::test_arbitrary_buffer: ok")
"###);
    assert_output(&out, r###"UnpackIteratorTest::test_arbitrary_buffer: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/struct/unpack_iterator_test__test_iterate.py`.
#[test]
fn test_gen_behavior_std_libs_struct_unpack_iterator_test__test_iterate() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct"
# dimension = "behavior"
# case = "unpack_iterator_test__test_iterate"
# subject = "cpython.test_struct.UnpackIteratorTest.test_iterate"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_struct.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_struct.py::UnpackIteratorTest::test_iterate
"""Auto-ported test: UnpackIteratorTest::test_iterate (CPython 3.12 oracle)."""


from collections import abc
import array
import gc
import math
import operator
import unittest
import struct
import sys
import weakref
from test import support
from test.support import import_helper
from test.support.script_helper import assert_python_ok


ISBIGENDIAN = sys.byteorder == 'big'

integer_codes = ('b', 'B', 'h', 'H', 'i', 'I', 'l', 'L', 'q', 'Q', 'n', 'N')

byteorders = ('', '@', '=', '<', '>', '!')

def iter_integer_formats(byteorders=byteorders):
    for code in integer_codes:
        for byteorder in byteorders:
            if byteorder not in ('', '@') and code in ('n', 'N'):
                continue
            yield (code, byteorder)

def string_reverse(s):
    return s[::-1]

def bigendian_to_native(value):
    if ISBIGENDIAN:
        return value
    else:
        return string_reverse(value)


# --- test body ---
s = struct.Struct('>IB')
b = bytes(range(1, 16))
it = s.iter_unpack(b)

assert next(it) == (16909060, 5)

assert next(it) == (101124105, 10)

assert next(it) == (185339150, 15)

try:
    next(it)
    raise AssertionError('expected StopIteration')
except StopIteration:
    pass

try:
    next(it)
    raise AssertionError('expected StopIteration')
except StopIteration:
    pass
print("UnpackIteratorTest::test_iterate: ok")
"###);
    assert_output(&out, r###"UnpackIteratorTest::test_iterate: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/struct/unpack_iterator_test__test_length_hint.py`.
#[test]
fn test_gen_behavior_std_libs_struct_unpack_iterator_test__test_length_hint() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct"
# dimension = "behavior"
# case = "unpack_iterator_test__test_length_hint"
# subject = "cpython.test_struct.UnpackIteratorTest.test_length_hint"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_struct.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_struct.py::UnpackIteratorTest::test_length_hint
"""Auto-ported test: UnpackIteratorTest::test_length_hint (CPython 3.12 oracle)."""


from collections import abc
import array
import gc
import math
import operator
import unittest
import struct
import sys
import weakref
from test import support
from test.support import import_helper
from test.support.script_helper import assert_python_ok


ISBIGENDIAN = sys.byteorder == 'big'

integer_codes = ('b', 'B', 'h', 'H', 'i', 'I', 'l', 'L', 'q', 'Q', 'n', 'N')

byteorders = ('', '@', '=', '<', '>', '!')

def iter_integer_formats(byteorders=byteorders):
    for code in integer_codes:
        for byteorder in byteorders:
            if byteorder not in ('', '@') and code in ('n', 'N'):
                continue
            yield (code, byteorder)

def string_reverse(s):
    return s[::-1]

def bigendian_to_native(value):
    if ISBIGENDIAN:
        return value
    else:
        return string_reverse(value)


# --- test body ---
lh = operator.length_hint
s = struct.Struct('>IB')
b = bytes(range(1, 16))
it = s.iter_unpack(b)

assert lh(it) == 3
next(it)

assert lh(it) == 2
next(it)

assert lh(it) == 1
next(it)

assert lh(it) == 0

try:
    next(it)
    raise AssertionError('expected StopIteration')
except StopIteration:
    pass

assert lh(it) == 0
print("UnpackIteratorTest::test_length_hint: ok")
"###);
    assert_output(&out, r###"UnpackIteratorTest::test_length_hint: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/struct/unpack_iterator_test__test_module_func.py`.
#[test]
fn test_gen_behavior_std_libs_struct_unpack_iterator_test__test_module_func() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct"
# dimension = "behavior"
# case = "unpack_iterator_test__test_module_func"
# subject = "cpython.test_struct.UnpackIteratorTest.test_module_func"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_struct.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_struct.py::UnpackIteratorTest::test_module_func
"""Auto-ported test: UnpackIteratorTest::test_module_func (CPython 3.12 oracle)."""


from collections import abc
import array
import gc
import math
import operator
import unittest
import struct
import sys
import weakref
from test import support
from test.support import import_helper
from test.support.script_helper import assert_python_ok


ISBIGENDIAN = sys.byteorder == 'big'

integer_codes = ('b', 'B', 'h', 'H', 'i', 'I', 'l', 'L', 'q', 'Q', 'n', 'N')

byteorders = ('', '@', '=', '<', '>', '!')

def iter_integer_formats(byteorders=byteorders):
    for code in integer_codes:
        for byteorder in byteorders:
            if byteorder not in ('', '@') and code in ('n', 'N'):
                continue
            yield (code, byteorder)

def string_reverse(s):
    return s[::-1]

def bigendian_to_native(value):
    if ISBIGENDIAN:
        return value
    else:
        return string_reverse(value)


# --- test body ---
it = struct.iter_unpack('>IB', bytes(range(1, 11)))

assert next(it) == (16909060, 5)

assert next(it) == (101124105, 10)

try:
    next(it)
    raise AssertionError('expected StopIteration')
except StopIteration:
    pass

try:
    next(it)
    raise AssertionError('expected StopIteration')
except StopIteration:
    pass
print("UnpackIteratorTest::test_module_func: ok")
"###);
    assert_output(&out, r###"UnpackIteratorTest::test_module_func: ok
"###);
}
