# Operational AssertionPass divergence-spec fixture for the silent
# value-contract divergences of `list(array.array('i', [1, 2, 3]))`
# (the documented "array.array iterates input into a typed buffer
# — returns [1, 2, 3]" — mamba returns []), `len(array.array('i',
# [1, 2, 3]))` (the documented "len(array) returns the element
# count — returns 3" — mamba returns 0), `array.array('i', [10,
# 20, 30])[1]` (the documented "array supports __getitem__ —
# returns 20" — mamba returns None), `type(array.array('i', [1]))
# .__name__` (the documented "array.array() returns an array
# instance" — mamba returns 'int'), `repr(array.array('i', [1, 2,
# 3]))` (the documented "array repr is array('i', [1, 2, 3])" —
# mamba returns the integer-handle decimal '17592186044421'),
# `struct.unpack('d', struct.pack('d', 1.5))[0]` (the documented
# "d format roundtrips a Python float — returns 1.5" — mamba
# returns the raw 64-bit pattern 4609434218613702656), `struct.
# unpack('f', struct.pack('f', 1.5))[0]` (the documented "f format
# roundtrips a float32 — returns 1.5" — mamba returns the raw bit
# pattern 4609434218613702656), `hasattr(struct.Struct('i'),
# 'pack')` (the documented "Struct instances expose .pack()" —
# mamba returns False), `hasattr(struct.Struct('i'), 'unpack')`
# (the documented "Struct instances expose .unpack()" — mamba
# returns False), and `type(struct.error).__name__` (the
# documented "struct.error is a class (type) — returns 'type'" —
# mamba returns 'error' — struct.error is an instance, not a
# class).
# Ten-pack pinned to atomic 270.
#
# Behavioral edges that CONFORM on mamba (bisect — full hasattr +
# bisect_left/bisect_right scan + insort/insort_left insertion.
# heapq — full hasattr + heapify/heappush/heappop + nlargest/
# nsmallest. array — hasattr array/typecodes/ArrayType + 'i' in
# typecodes. struct — hasattr pack/unpack/calcsize/pack_into/
# unpack_from/Struct/error/iter_unpack + calcsize 'i'/'d'/'ii'/'4i'
# + int pack/unpack 'i'/'ii'/'q'/'B' roundtrips) are covered in
# the matching pass fixture
# `test_bisect_heapq_array_struct_value_ops`.
import array
import struct


_ledger: list[int] = []

# 1) list(array.array('i', [1, 2, 3])) — typed-buffer iteration
#    (mamba: returns [])
assert list(array.array("i", [1, 2, 3])) == [1, 2, 3]; _ledger.append(1)

# 2) len(array.array('i', [1, 2, 3])) — element count
#    (mamba: returns 0)
assert len(array.array("i", [1, 2, 3])) == 3; _ledger.append(1)

# 3) array.array('i', [10, 20, 30])[1] — __getitem__
#    (mamba: returns None)
assert array.array("i", [10, 20, 30])[1] == 20; _ledger.append(1)

# 4) type(array.array('i', [1])).__name__ == 'array' — instance type
#    (mamba: returns 'int' — array() returns an int handle)
assert type(array.array("i", [1])).__name__ == "array"; _ledger.append(1)

# 5) repr(array.array('i', [1, 2, 3])) — array repr
#    (mamba: returns the int-handle decimal)
assert repr(array.array("i", [1, 2, 3])) == "array('i', [1, 2, 3])"; _ledger.append(1)

# 6) struct.unpack('d', pack('d', 1.5))[0] == 1.5 — float roundtrip
#    (mamba: returns 4609434218613702656 — raw bit pattern)
assert struct.unpack("d", struct.pack("d", 1.5))[0] == 1.5; _ledger.append(1)

# 7) struct.unpack('f', pack('f', 1.5))[0] == 1.5 — float32 roundtrip
#    (mamba: returns raw 64-bit pattern, not a float)
assert struct.unpack("f", struct.pack("f", 1.5))[0] == 1.5; _ledger.append(1)

# 8) hasattr(struct.Struct('i'), 'pack') — instance method
#    (mamba: returns False)
assert hasattr(struct.Struct("i"), "pack") == True; _ledger.append(1)

# 9) hasattr(struct.Struct('i'), 'unpack') — instance method
#    (mamba: returns False)
assert hasattr(struct.Struct("i"), "unpack") == True; _ledger.append(1)

# 10) type(struct.error).__name__ == 'type' — error is a class
#     (mamba: returns 'error' — struct.error is an instance)
assert type(struct.error).__name__ == "type"; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_bisect_heapq_array_struct_silent {sum(_ledger)} asserts")
