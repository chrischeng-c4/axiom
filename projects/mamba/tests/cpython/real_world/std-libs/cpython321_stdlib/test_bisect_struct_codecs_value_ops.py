# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_bisect_struct_codecs_value_ops"
# subject = "cpython321.test_bisect_struct_codecs_value_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_bisect_struct_codecs_value_ops.py"
# status = "filled"
# ///
"""cpython321.test_bisect_struct_codecs_value_ops: execute CPython 3.12 seed test_bisect_struct_codecs_value_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for the value contract of three
# wire-format codecs used by every binary protocol implementation:
# `bisect` (sorted-list search and insertion used by every priority
# queue / interval tree), `struct` (fixed-width integer packing
# layer used by every binary file format / network protocol),
# `codecs` (text/byte transform layer + BOM constants used by every
# text-file decoder). No fixture coverage yet for any of the three
# at this level of detail.
#
# The matching subset between mamba and CPython is the *byte-exact*
# transform layer: bisect.bisect_left / bisect_right / bisect /
# insort_left / insort_right / insort all return the documented
# integer positions and mutate lists in place; struct.pack /
# struct.unpack / struct.calcsize all reproduce the documented
# little-endian and big-endian byte layouts; codecs.encode /
# codecs.decode round-trip utf-8 text; the documented BOM_UTF8 /
# BOM_UTF16_LE / BOM_UTF16_BE / BOM_UTF32_LE byte constants are
# byte-exact; struct.error class-name identity ("error") matches.
#
# Surface in this fixture:
#   • bisect.bisect_left([1,2,3,4,5], 3) == 2;
#   • bisect.bisect_right([1,2,3,4,5], 3) == 3;
#   • bisect.bisect([1,2,3,4,5], 3) == 3;
#   • bisect.bisect_left([], 5) == 0 (empty input);
#   • bisect.bisect_right([], 5) == 0 (empty input);
#   • bisect.insort([1,2,4,5], 3) leaves [1,2,3,4,5];
#   • bisect.insort_left([1,2,4,5], 3) leaves [1,2,3,4,5];
#   • bisect.insort_right([1,2,4,5], 3) leaves [1,2,3,4,5];
#   • struct.pack('i', 1234) == b'\\xd2\\x04\\x00\\x00' (native LE);
#   • struct.pack('>I', 1234) == b'\\x00\\x00\\x04\\xd2' (big-endian);
#   • struct.unpack('i', struct.pack('i', 1234)) == (1234,);
#   • struct.pack('>HH', 1, 2) == b'\\x00\\x01\\x00\\x02';
#   • struct.unpack('>HH', b'\\x00\\x01\\x00\\x02') == (1, 2);
#   • struct.calcsize('i') == 4 / 'd' == 8 / 'Q' == 8 / 'B' == 1 /
#     'h' == 2 (documented fixed-width sentinels);
#   • struct.error.__name__ == "error" (bare class-name identity);
#   • codecs.encode("hello", "utf-8") == b"hello";
#   • codecs.decode(b"hello", "utf-8") == "hello";
#   • codecs.BOM_UTF8 == b"\\xef\\xbb\\xbf";
#   • codecs.BOM_UTF16_LE == b"\\xff\\xfe";
#   • codecs.BOM_UTF16_BE == b"\\xfe\\xff";
#   • codecs.BOM_UTF32_LE == b"\\xff\\xfe\\x00\\x00";
#   • array.typecodes == "bBuhHiIlLqQfd" (sentinel string);
#   • array.array('i', ...).typecode == "i" (typecode attribute);
#   • array.array('i', ...).itemsize == 4 (itemsize attribute).
#
# Behavioral edges that DIVERGE on mamba (array.array() returning
# an int handle instead of a real array — len/subscript/list-iter
# all degrade, codecs.lookup returning a tuple instead of CodecInfo
# with .name == "utf-8", struct.error being an exception instance
# rather than a type) are covered in the divergence-spec fixture
# `lang_array_codecinfo_struct_error_silent`.
import bisect
import struct
import codecs
import array

_ledger: list[int] = []

# 1) bisect — bisect_left / bisect_right / bisect on a sorted list
assert bisect.bisect_left([1, 2, 3, 4, 5], 3) == 2; _ledger.append(1)
assert bisect.bisect_right([1, 2, 3, 4, 5], 3) == 3; _ledger.append(1)
assert bisect.bisect([1, 2, 3, 4, 5], 3) == 3; _ledger.append(1)
assert bisect.bisect_left([1, 2, 3, 4, 5], 0) == 0; _ledger.append(1)
assert bisect.bisect_right([1, 2, 3, 4, 5], 10) == 5; _ledger.append(1)

# 2) bisect — empty input edge
assert bisect.bisect_left([], 5) == 0; _ledger.append(1)
assert bisect.bisect_right([], 5) == 0; _ledger.append(1)

# 3) bisect — insort / insort_left / insort_right mutate in place
_la: list[int] = [1, 2, 4, 5]
bisect.insort(_la, 3)
assert _la == [1, 2, 3, 4, 5]; _ledger.append(1)
_lb: list[int] = [1, 2, 4, 5]
bisect.insort_left(_lb, 3)
assert _lb == [1, 2, 3, 4, 5]; _ledger.append(1)
_lc: list[int] = [1, 2, 4, 5]
bisect.insort_right(_lc, 3)
assert _lc == [1, 2, 3, 4, 5]; _ledger.append(1)

# 4) struct.pack — native int / big-endian uint
assert struct.pack('i', 1234) == b'\xd2\x04\x00\x00'; _ledger.append(1)
assert struct.pack('>I', 1234) == b'\x00\x00\x04\xd2'; _ledger.append(1)

# 5) struct.unpack — round-trip the int
assert struct.unpack('i', struct.pack('i', 1234)) == (1234,); _ledger.append(1)
assert struct.unpack('>I', b'\x00\x00\x04\xd2') == (1234,); _ledger.append(1)

# 6) struct.pack / unpack — multi-value big-endian
assert struct.pack('>HH', 1, 2) == b'\x00\x01\x00\x02'; _ledger.append(1)
assert struct.unpack('>HH', b'\x00\x01\x00\x02') == (1, 2); _ledger.append(1)

# 7) struct.calcsize — fixed-width sentinels
assert struct.calcsize('i') == 4; _ledger.append(1)
assert struct.calcsize('d') == 8; _ledger.append(1)
assert struct.calcsize('Q') == 8; _ledger.append(1)
assert struct.calcsize('B') == 1; _ledger.append(1)
assert struct.calcsize('h') == 2; _ledger.append(1)

# 8) struct.error — bare class-name identity (matches on both)
assert struct.error.__name__ == "error"; _ledger.append(1)

# 9) codecs.encode / decode — utf-8 round-trip
assert codecs.encode("hello", "utf-8") == b"hello"; _ledger.append(1)
assert codecs.decode(b"hello", "utf-8") == "hello"; _ledger.append(1)
assert codecs.encode("alpha", "utf-8") == b"alpha"; _ledger.append(1)
assert codecs.decode(b"alpha", "utf-8") == "alpha"; _ledger.append(1)

# 10) codecs BOM constants — byte-exact
assert codecs.BOM_UTF8 == b'\xef\xbb\xbf'; _ledger.append(1)
assert codecs.BOM_UTF16_LE == b'\xff\xfe'; _ledger.append(1)
assert codecs.BOM_UTF16_BE == b'\xfe\xff'; _ledger.append(1)
assert codecs.BOM_UTF32_LE == b'\xff\xfe\x00\x00'; _ledger.append(1)

# 11) array.typecodes — documented sentinel string
assert array.typecodes == "bBuhHiIlLqQfd"; _ledger.append(1)

# 12) array.array — typecode / itemsize attributes match the input
_ai = array.array('i', [1, 2, 3])
assert _ai.typecode == "i"; _ledger.append(1)
assert _ai.itemsize == 4; _ledger.append(1)
_ad = array.array('d', [1.0, 2.0])
assert _ad.typecode == "d"; _ledger.append(1)
assert _ad.itemsize == 8; _ledger.append(1)
_ab = array.array('B', [1, 2])
assert _ab.typecode == "B"; _ledger.append(1)
assert _ab.itemsize == 1; _ledger.append(1)

# 13) hasattr surface — module-level helpers
assert hasattr(bisect, "bisect_left"); _ledger.append(1)
assert hasattr(bisect, "bisect_right"); _ledger.append(1)
assert hasattr(bisect, "insort"); _ledger.append(1)
assert hasattr(struct, "pack"); _ledger.append(1)
assert hasattr(struct, "unpack"); _ledger.append(1)
assert hasattr(struct, "calcsize"); _ledger.append(1)
assert hasattr(codecs, "encode"); _ledger.append(1)
assert hasattr(codecs, "decode"); _ledger.append(1)
assert hasattr(codecs, "BOM_UTF8"); _ledger.append(1)

# NB: array.array() returning a real array (len / subscript /
# list-iter), codecs.lookup returning CodecInfo with .name=="utf-8",
# struct.error being a type rather than an instance — all DIVERGE
# on mamba and live in `lang_array_codecinfo_struct_error_silent`.

print(f"MAMBA_ASSERTION_PASS: test_bisect_struct_codecs_value_ops {sum(_ledger)} asserts")
