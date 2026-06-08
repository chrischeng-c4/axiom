# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_io_array_zlib_silent"
# subject = "cpython321.lang_io_array_zlib_silent"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_io_array_zlib_silent.py"
# status = "filled"
# ///
"""cpython321.lang_io_array_zlib_silent: execute CPython 3.12 seed lang_io_array_zlib_silent"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Operational AssertionPass divergence-spec fixture for the silent
# value-contract divergences of `type(io.StringIO('hello')).__name__
# == 'StringIO'` (the documented "StringIO() returns a StringIO
# instance" — mamba returns 'dict' — constructor degrades to a
# plain dict), `io.StringIO('hello').read() == 'hello'` (the
# documented "StringIO('hello').read() returns the seed contents" —
# mamba returns '' — initial value is not stored on construction),
# `io.StringIO('hello').getvalue() == 'hello'` (the documented
# "StringIO('hello').getvalue() returns the seed contents" — mamba
# returns '' — initial value is not stored on construction),
# `type(io.BytesIO(b'hello')).__name__ == 'BytesIO'` (the documented
# "BytesIO() returns a BytesIO instance" — mamba returns 'dict' —
# constructor degrades to a plain dict), `io.BytesIO(b'hello').
# getvalue() == b'hello'` (the documented "BytesIO(b'hello').
# getvalue() returns the seed contents" — mamba returns b'' —
# initial value is not stored on construction), `array.array('i',
# [1, 2, 3])[0] == 1` (the documented "array typed-buffer indexing
# yields the integer at that slot" — mamba returns None — indexing
# the handle-typed sentinel produces None), `array.array('i', [1, 2
# , 3]).typecode == 'i'` (the documented "array.typecode echoes the
# constructor typecode" — mamba returns None — attribute resolves to
# None placeholder), `array.array('i', [1, 2, 3]).itemsize == 4`
# (the documented "array.itemsize is the byte width of one slot" —
# mamba returns None — attribute resolves to None placeholder),
# `hasattr(zlib, 'Z_BEST_SPEED')` (the documented "zlib exposes the
# Z_BEST_SPEED compression-level constant" — mamba returns False),
# and `hasattr(zlib, 'error')` (the documented "zlib exposes the
# error exception class" — mamba returns False).
# Ten-pack pinned to atomic 298.
#
# Behavioral edges that CONFORM on mamba (struct — hasattr pack/
# unpack/pack_into/unpack_from/calcsize/Struct/error/iter_unpack +
# pack/unpack big-endian + calcsize/Struct. zlib — hasattr compress/
# decompress/crc32/adler32 + crc32/adler32 known-vectors + compress/
# decompress round-trip. array — hasattr array/typecodes. io —
# hasattr StringIO/BytesIO) are covered in the matching pass fixture
# `test_struct_zlib_value_ops`.
import io
import array
import zlib


_ledger: list[int] = []

# 1) type(io.StringIO('hello')).__name__ == 'StringIO' — StringIO instance
#    (mamba: returns 'dict' — constructor degrades to plain dict)
assert type(io.StringIO("hello")).__name__ == "StringIO"; _ledger.append(1)

# 2) io.StringIO('hello').read() == 'hello' — seed contents readable
#    (mamba: returns '' — initial value not stored on construction)
assert io.StringIO("hello").read() == "hello"; _ledger.append(1)

# 3) io.StringIO('hello').getvalue() == 'hello' — seed contents via getvalue
#    (mamba: returns '' — initial value not stored on construction)
assert io.StringIO("hello").getvalue() == "hello"; _ledger.append(1)

# 4) type(io.BytesIO(b'hello')).__name__ == 'BytesIO' — BytesIO instance
#    (mamba: returns 'dict' — constructor degrades to plain dict)
assert type(io.BytesIO(b"hello")).__name__ == "BytesIO"; _ledger.append(1)

# 5) io.BytesIO(b'hello').getvalue() == b'hello' — seed contents via getvalue
#    (mamba: returns b'' — initial value not stored on construction)
assert io.BytesIO(b"hello").getvalue() == b"hello"; _ledger.append(1)

# 6) array.array('i', [1,2,3])[0] == 1 — typed-buffer indexing
#    (mamba: returns None — indexing handle-typed sentinel produces None)
assert array.array("i", [1, 2, 3])[0] == 1; _ledger.append(1)

# 7) array.array('i', [1,2,3]).typecode == 'i' — typecode echo
#    (mamba: returns None — attribute resolves to None placeholder)
assert array.array("i", [1, 2, 3]).typecode == "i"; _ledger.append(1)

# 8) array.array('i', [1,2,3]).itemsize == 4 — byte width per slot
#    (mamba: returns None — attribute resolves to None placeholder)
assert array.array("i", [1, 2, 3]).itemsize == 4; _ledger.append(1)

# 9) hasattr(zlib, 'Z_BEST_SPEED') — Z_BEST_SPEED level constant
#    (mamba: returns False)
assert hasattr(zlib, "Z_BEST_SPEED") == True; _ledger.append(1)

# 10) hasattr(zlib, 'error') — zlib error exception class
#     (mamba: returns False)
assert hasattr(zlib, "error") == True; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_io_array_zlib_silent {sum(_ledger)} asserts")
