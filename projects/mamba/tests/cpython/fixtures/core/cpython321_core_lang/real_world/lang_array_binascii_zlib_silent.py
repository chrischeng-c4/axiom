# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_array_binascii_zlib_silent"
# subject = "cpython321.lang_array_binascii_zlib_silent"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_array_binascii_zlib_silent.py"
# status = "filled"
# ///
"""cpython321.lang_array_binascii_zlib_silent: execute CPython 3.12 seed lang_array_binascii_zlib_silent"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Operational AssertionPass divergence-spec fixture for the silent
# value-contract divergences of `type(array.array('i', [1,2,3])).
# __name__` (the documented "array.array() returns an 'array'
# instance" — mamba returns 'int' — opaque int handle), `len(array.
# array('i', [1,2,3]))` (the documented "len of 3-element array is
# 3" — mamba returns 0 — len of opaque handle), `hasattr(binascii,
# 'crc32')` (the documented "binascii exposes the crc32 helper" —
# mamba returns False), `hasattr(binascii, 'Error')` (the documented
# "binascii exposes the Error exception" — mamba returns False),
# `hasattr(binascii, 'Incomplete')` (the documented "binascii
# exposes the Incomplete exception" — mamba returns False), `hasattr
# (zlib, 'compressobj')` (the documented "zlib exposes the
# compressobj factory" — mamba returns False), `hasattr(zlib,
# 'decompressobj')` (the documented "zlib exposes the decompressobj
# factory" — mamba returns False), `hasattr(zlib, 'Z_DEFAULT_
# COMPRESSION')` (the documented "zlib exposes the Z_DEFAULT_
# COMPRESSION constant" — mamba returns False), `zlib.Z_BEST_
# COMPRESSION == 9` (the documented "Z_BEST_COMPRESSION constant
# value is 9" — mamba returns None), and `zlib.DEFLATED == 8` (the
# documented "DEFLATED constant value is 8" — mamba returns None).
# Ten-pack pinned to atomic 288.
#
# Behavioral edges that CONFORM on mamba (array — hasattr array/
# typecodes + typecodes string + tolist/typecode/itemsize. struct —
# full hasattr + pack bytes + calcsize 4/1/2/8/4 + unpack tuple +
# round-trip. binascii — hasattr hexlify/unhexlify/a2b_hex/b2a_hex/
# a2b_base64/b2a_base64 + hexlify/unhexlify. zlib — hasattr compress/
# decompress/crc32/adler32 + crc32/adler32 + round-trip) are covered
# in the matching pass fixture `test_array_struct_binascii_zlib_
# value_ops`.
import array
import binascii
import zlib


_ledger: list[int] = []

# 1) type(array.array('i', [1,2,3])).__name__ == 'array' — instance type
#    (mamba: returns 'int' — opaque int handle)
assert type(array.array("i", [1, 2, 3])).__name__ == "array"; _ledger.append(1)

# 2) len(array.array('i', [1,2,3])) == 3 — len of 3-element array
#    (mamba: returns 0 — len of opaque handle)
assert len(array.array("i", [1, 2, 3])) == 3; _ledger.append(1)

# 3) hasattr(binascii, 'crc32') — crc32 checksum helper
#    (mamba: returns False)
assert hasattr(binascii, "crc32") == True; _ledger.append(1)

# 4) hasattr(binascii, 'Error') — Error exception class
#    (mamba: returns False)
assert hasattr(binascii, "Error") == True; _ledger.append(1)

# 5) hasattr(binascii, 'Incomplete') — Incomplete exception class
#    (mamba: returns False)
assert hasattr(binascii, "Incomplete") == True; _ledger.append(1)

# 6) hasattr(zlib, 'compressobj') — incremental compressor factory
#    (mamba: returns False)
assert hasattr(zlib, "compressobj") == True; _ledger.append(1)

# 7) hasattr(zlib, 'decompressobj') — incremental decompressor factory
#    (mamba: returns False)
assert hasattr(zlib, "decompressobj") == True; _ledger.append(1)

# 8) hasattr(zlib, 'Z_DEFAULT_COMPRESSION') — default-level constant
#    (mamba: returns False)
assert hasattr(zlib, "Z_DEFAULT_COMPRESSION") == True; _ledger.append(1)

# 9) zlib.Z_BEST_COMPRESSION == 9 — best-compression constant value
#    (mamba: returns None)
assert zlib.Z_BEST_COMPRESSION == 9; _ledger.append(1)

# 10) zlib.DEFLATED == 8 — DEFLATED method constant value
#     (mamba: returns None)
assert zlib.DEFLATED == 8; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_array_binascii_zlib_silent {sum(_ledger)} asserts")
