# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "cpython321_builtins"
# dimension = "real_world"
# case = "test_memoryview_struct_range_ipaddress_value_ops"
# subject = "cpython321.test_memoryview_struct_range_ipaddress_value_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_memoryview_struct_range_ipaddress_value_ops.py"
# status = "filled"
# ///
"""cpython321.test_memoryview_struct_range_ipaddress_value_ops: execute CPython 3.12 seed test_memoryview_struct_range_ipaddress_value_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Atomic 252 pass conformance — memoryview deep surface
# (len/index/tobytes/readonly/format/itemsize/nbytes/ndim/equality/
# slice/tolist) + bytearray internal correctness (repr after append,
# len+index after append) + struct module (hasattr surface
# pack/unpack/calcsize/pack_into/unpack_from/Struct/error + pack
# int16/unpack int16/calcsize/pack int32/unpack int32/pack double/
# unpack double round-trips on big-endian) + range edges
# (len/index/slice/negative-step iteration/membership in/not-in/
# .count/equality across two range views) + ipaddress module
# (IPv4Address/IPv6Address/IPv4Network/ip_address hasattr surface +
# .version for IPv4 and IPv6) + number tower (int.bit_length/
# int.to_bytes/int.from_bytes/float.is_integer/bool-int arithmetic/
# divmod tuple/round-half-even on 2.5/round-half-even on 0.5). All
# asserts match between CPython 3.12 and mamba.
import struct
import ipaddress


_ledger: list[int] = []

# 1) memoryview deep surface
assert len(memoryview(b"hello")) == 5; _ledger.append(1)
assert memoryview(b"hello")[0] == 104; _ledger.append(1)
assert memoryview(b"hello").tobytes() == b"hello"; _ledger.append(1)
assert memoryview(b"hello").readonly == True; _ledger.append(1)
assert memoryview(b"hello").format == "B"; _ledger.append(1)
assert memoryview(b"hello").itemsize == 1; _ledger.append(1)
assert memoryview(b"hello").nbytes == 5; _ledger.append(1)
assert memoryview(b"hello").ndim == 1; _ledger.append(1)
assert (memoryview(b"hi") == b"hi") == True; _ledger.append(1)
assert bytes(memoryview(b"hello")[1:4]) == b"ell"; _ledger.append(1)
assert memoryview(b"hi").tolist() == [104, 105]; _ledger.append(1)

# 2) bytearray internal correctness
def _ba_repr_after_append() -> str:
    b = bytearray(b"hi")
    b.append(33)
    return repr(b)
assert _ba_repr_after_append() == "bytearray(b'hi!')"; _ledger.append(1)

def _ba_index_after_append():
    b = bytearray(b"hi")
    b.append(33)
    return (len(b), b[0], b[1], b[2])
assert _ba_index_after_append() == (3, 104, 105, 33); _ledger.append(1)

# 3) struct module hasattr surface
assert hasattr(struct, "pack") == True; _ledger.append(1)
assert hasattr(struct, "unpack") == True; _ledger.append(1)
assert hasattr(struct, "calcsize") == True; _ledger.append(1)
assert hasattr(struct, "pack_into") == True; _ledger.append(1)
assert hasattr(struct, "unpack_from") == True; _ledger.append(1)
assert hasattr(struct, "Struct") == True; _ledger.append(1)
assert hasattr(struct, "error") == True; _ledger.append(1)

# 4) struct numeric pack/unpack big-endian
assert struct.pack(">h", 1234) == b"\x04\xd2"; _ledger.append(1)
assert struct.unpack(">h", b"\x04\xd2") == (1234,); _ledger.append(1)
assert struct.calcsize(">h") == 2; _ledger.append(1)
assert struct.pack(">i", 65535) == b"\x00\x00\xff\xff"; _ledger.append(1)
assert struct.unpack(">i", b"\x00\x00\xff\xff") == (65535,); _ledger.append(1)
assert struct.pack(">d", 1.5) == b"?\xf8\x00\x00\x00\x00\x00\x00"; _ledger.append(1)
assert struct.unpack(">d", b"?\xf8\x00\x00\x00\x00\x00\x00") == (1.5,); _ledger.append(1)

# 5) range edges — len, index, slice, neg step, membership, count, eq
assert len(range(10)) == 10; _ledger.append(1)
assert range(10)[3] == 3; _ledger.append(1)
assert list(range(10)[2:5]) == [2, 3, 4]; _ledger.append(1)
assert list(range(5, 0, -1)) == [5, 4, 3, 2, 1]; _ledger.append(1)
assert (5 in range(10)) == True; _ledger.append(1)
assert (99 in range(10)) == False; _ledger.append(1)
assert range(10).count(5) == 1; _ledger.append(1)
assert (range(10) == range(10)) == True; _ledger.append(1)
assert (range(0, 10) == range(0, 10, 1)) == True; _ledger.append(1)

# 6) ipaddress hasattr + .version
assert hasattr(ipaddress, "IPv4Address") == True; _ledger.append(1)
assert hasattr(ipaddress, "IPv6Address") == True; _ledger.append(1)
assert hasattr(ipaddress, "IPv4Network") == True; _ledger.append(1)
assert hasattr(ipaddress, "ip_address") == True; _ledger.append(1)
assert ipaddress.ip_address("192.168.1.1").version == 4; _ledger.append(1)
assert ipaddress.ip_address("::1").version == 6; _ledger.append(1)

# 7) number tower
assert (255).bit_length() == 8; _ledger.append(1)
assert (255).to_bytes(2, "big") == b"\x00\xff"; _ledger.append(1)
assert int.from_bytes(b"\x00\xff", "big") == 255; _ledger.append(1)
assert (1.0).is_integer() == True; _ledger.append(1)
assert int(True) + int(False) == 1; _ledger.append(1)
assert divmod(10, 3) == (3, 1); _ledger.append(1)
assert round(2.5) == 2; _ledger.append(1)
assert round(0.5) == 0; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_memoryview_struct_range_ipaddress_value_ops {sum(_ledger)} asserts")
