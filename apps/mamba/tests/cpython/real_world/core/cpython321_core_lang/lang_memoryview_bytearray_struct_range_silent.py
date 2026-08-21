# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_memoryview_bytearray_struct_range_silent"
# subject = "cpython321.lang_memoryview_bytearray_struct_range_silent"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_memoryview_bytearray_struct_range_silent.py"
# status = "filled"
# ///
"""cpython321.lang_memoryview_bytearray_struct_range_silent: execute CPython 3.12 seed lang_memoryview_bytearray_struct_range_silent"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Operational AssertionPass divergence-spec fixture for the silent
# value-contract divergences of `bytes(memoryview(...))` whole-view
# conversion (the documented "bytes(mv) materializes the underlying
# buffer into a new bytes object" — mamba returns b'' regardless of
# content), `memoryview.shape` (the documented "shape is a tuple
# describing the dimensions, e.g. (5,) for a 1-D 5-byte buffer" —
# mamba returns None), `memoryview.hex()` (the documented "hex()
# returns the buffer formatted as a hex string" — mamba raises
# AttributeError), `bytes(bytearray(...))` conversion (the documented
# "bytes(ba) materializes a bytearray into a new immutable bytes" —
# mamba returns b'' even though the bytearray internally holds the
# correct content), `struct.pack("3s", b"abc")` fixed-string packing
# (the documented "the `s` format produces the bytes argument padded
# or truncated to the size prefix" — mamba returns b'a\x00\x00'),
# `range(start, stop).start` (the documented "range objects expose
# .start/.stop/.step attributes" — mamba returns None for all three),
# `range.index(x)` lookup (the documented "range objects expose
# .index returning the position of the first matching element" —
# mamba raises AttributeError because range objects are int handles),
# `str(ipaddress.ip_address("192.168.1.1"))` (the documented
# "ip_address(str) -> IPvNAddress whose str is the canonical address"
# — mamba returns the boxed-handle integer as a string), `float.hex()`
# representation (the documented "float.hex returns '0x1.0000000000000p+0'
# style hexadecimal float representation" — mamba returns the raw
# IEEE-754 bit pattern '0x3ff0000000000000'), and `abs(complex)`
# (the documented "abs() on a complex number returns the float
# magnitude sqrt(re^2+im^2)" — mamba returns a boxed-handle int).
# Ten-pack pinned to atomic 252.
#
# Behavioral edges that CONFORM on mamba (memoryview surface
# len/index/tobytes/readonly/format/itemsize/nbytes/ndim/equality-
# against-bytes/slice-then-bytes/tolist; bytearray repr after append
# + len+index after append; struct hasattr surface
# pack/unpack/calcsize/pack_into/unpack_from/Struct/error +
# numeric pack/unpack/calcsize for >h / >i / >d round-trips;
# range edges len/index/slice/negative-step list/in/not-in/.count/
# range==range / range==range-with-explicit-step-1; ipaddress
# hasattr IPv4Address/IPv6Address/IPv4Network/ip_address + .version
# for IPv4 and IPv6; number tower int.bit_length/int.to_bytes/
# int.from_bytes/float.is_integer/bool+int arithmetic/divmod/
# round-half-even 2.5 and 0.5) are covered in the matching pass
# fixture
# `test_memoryview_struct_range_ipaddress_value_ops`.
import struct
import ipaddress
from typing import Any


_ledger: list[int] = []

# 1) bytes(memoryview(...)) — whole-view conversion to bytes
#    (mamba: returns b'' regardless of the underlying buffer)
assert bytes(memoryview(b"hello")) == b"hello"; _ledger.append(1)

# 2) memoryview.shape — 1-D buffer reports (len,)
#    (mamba: returns None)
assert memoryview(b"hello").shape == (5,); _ledger.append(1)

# 3) memoryview.hex() — hex representation of buffer bytes
#    (mamba: AttributeError 'memoryview' object has no attribute 'hex')
def _mv_hex() -> Any:
    try:
        return memoryview(b"hi").hex()
    except AttributeError:
        return None
assert _mv_hex() == "6869"; _ledger.append(1)

# 4) bytes(bytearray(...)) — materialize bytearray into bytes
#    (mamba: returns b'' even though bytearray internal state is correct)
assert bytes(bytearray(b"hi")) == b"hi"; _ledger.append(1)

# 5) struct.pack("3s", b"abc") — fixed-length string packing
#    (mamba: returns b'a\x00\x00' instead of b'abc')
assert struct.pack("3s", b"abc") == b"abc"; _ledger.append(1)

# 6) range(2, 10).start — range objects expose .start attribute
#    (mamba: returns None because range is a boxed integer handle)
assert range(2, 10).start == 2; _ledger.append(1)

# 7) range(2, 10).index(5) — range exposes .index lookup method
#    (mamba: AttributeError 'int' object has no attribute 'index')
def _range_index() -> Any:
    try:
        return range(2, 10).index(5)
    except AttributeError:
        return None
assert _range_index() == 3; _ledger.append(1)

# 8) str(ip_address(...)) — canonical IPv4 string representation
#    (mamba: returns the boxed-handle integer stringified, e.g.
#    '4398046511104' instead of '192.168.1.1')
assert str(ipaddress.ip_address("192.168.1.1")) == "192.168.1.1"; _ledger.append(1)

# 9) float.hex() — '0x1.MANTISSAp+EXP' formatted hexadecimal repr
#    (mamba: returns the raw 64-bit IEEE-754 bit pattern hex,
#    e.g. '0x3ff0000000000000' for 1.0)
assert (1.0).hex() == "0x1.0000000000000p+0"; _ledger.append(1)

# 10) abs(complex) — float magnitude sqrt(re^2 + im^2)
#     (mamba: returns a boxed-handle int)
def _complex_abs() -> Any:
    return abs(3 + 4j)
assert _complex_abs() == 5.0; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_memoryview_bytearray_struct_range_silent {sum(_ledger)} asserts")
