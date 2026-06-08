# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array"
# dimension = "real_world"
# case = "typed_buffer_pipeline"
# subject = "array.array"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_array.py"
# status = "filled"
# ///
"""array.array: a numeric tool builds a typed array from a bytes buffer, runs in-place mutations (append/insert/pop/remove/reverse/byteswap), queries metadata (typecode/itemsize/buffer_info/count/index), extends and fromlists, and serializes back to bytes, asserting a deterministic aggregate at each step"""
import array

# Build an int32 array from a deterministic byte buffer (32 elements).
src = bytes([(i * 17 + 3) & 0xFF for i in range(128)])  # 32 int32 LE
a = array.array("i")
a.frombytes(src)
assert a.typecode == "i", f"typecode = {a.typecode!r}"
assert a.itemsize == 4, f"itemsize = {a.itemsize!r}"
_ptr, n = a.buffer_info()
assert n == 32, f"buffer_info_len = {n!r}"

# Mutations.
a.append(42)
a.append(99)
a.insert(0, -1)
popped = a.pop(-1)
assert popped == 99, f"popped = {popped!r}"

# Lookups.
assert a.count(42) == 1, f"count_42 = {a.count(42)!r}"
assert a.index(42) == 33, f"index_42 = {a.index(42)!r}"

# Round-trip via tobytes/frombytes.
out = a.tobytes()
assert len(out) == 136, f"tobytes_len = {len(out)!r}"
b = array.array("i")
b.frombytes(out)
_ptr_b, n_b = b.buffer_info()
assert n_b == 34, f"roundtrip_len = {n_b!r}"

# Reverse + byteswap on a 'H' array.
small = array.array("H")
small.frombytes(bytes([0x01, 0x02, 0x03, 0x04]))
small.byteswap()
assert small.tobytes().hex() == "02010403", f"byteswap = {small.tobytes().hex()!r}"
small.reverse()
assert small.tobytes().hex() == "04030201", f"reverse = {small.tobytes().hex()!r}"

# Remove first occurrence.
c = array.array("b")
c.frombytes(bytes([1, 2, 3, 2, 4]))
c.remove(2)
assert c.tobytes().hex() == "01030204", f"after_remove = {c.tobytes().hex()!r}"

# Extend from another array.
d = array.array("i")
d.frombytes(bytes([10, 0, 0, 0, 20, 0, 0, 0]))  # [10, 20]
e = array.array("i")
e.frombytes(bytes([30, 0, 0, 0, 40, 0, 0, 0]))  # [30, 40]
d.extend(e)
_ptr_d, n_d = d.buffer_info()
assert n_d == 4, f"extended_len = {n_d!r}"

# fromlist round-trip.
f = array.array("i")
f.fromlist([7, 11, 13])
_ptr_f, n_f = f.buffer_info()
assert n_f == 3, f"fromlist_len = {n_f!r}"

# Module-level constants.
assert len(array.typecodes) == 13, f"typecodes_len = {len(array.typecodes)!r}"

print("typed_buffer_pipeline OK")
