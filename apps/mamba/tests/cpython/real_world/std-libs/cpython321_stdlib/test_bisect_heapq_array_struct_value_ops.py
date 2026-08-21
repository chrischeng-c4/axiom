# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_bisect_heapq_array_struct_value_ops"
# subject = "cpython321.test_bisect_heapq_array_struct_value_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_bisect_heapq_array_struct_value_ops.py"
# status = "filled"
# ///
"""cpython321.test_bisect_heapq_array_struct_value_ops: execute CPython 3.12 seed test_bisect_heapq_array_struct_value_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Atomic 270 pass conformance — bisect module (hasattr bisect/bisect_
# left/bisect_right/insort/insort_left/insort_right + bisect_left/
# bisect_right scan positions + insort/insort_left in-place insertion)
# + heapq module (hasattr heappush/heappop/heapify/heapreplace/
# heappushpop/nlargest/nsmallest/merge + heapify/heappush/heappop +
# nlargest/nsmallest n=3) + array module (hasattr array/typecodes/
# ArrayType + 'i' in typecodes) + struct module (hasattr pack/unpack/
# calcsize/pack_into/unpack_from/Struct/error/iter_unpack + calcsize
# 'i'==4 / 'd'==8 / 'ii'==8 / '4i'==16 + pack 'i' 5 + unpack 'i'
# roundtrip + pack/unpack 'ii' roundtrip + pack/unpack 'q' roundtrip
# + pack/unpack 'B' roundtrip).
# All asserts match between CPython 3.12 and mamba.
import bisect
import heapq
import array
import struct


_ledger: list[int] = []

# 1) bisect — hasattr surface
assert hasattr(bisect, "bisect") == True; _ledger.append(1)
assert hasattr(bisect, "bisect_left") == True; _ledger.append(1)
assert hasattr(bisect, "bisect_right") == True; _ledger.append(1)
assert hasattr(bisect, "insort") == True; _ledger.append(1)
assert hasattr(bisect, "insort_left") == True; _ledger.append(1)
assert hasattr(bisect, "insort_right") == True; _ledger.append(1)

# 2) bisect — bisect_left/right scan positions
assert bisect.bisect_left([1, 2, 3, 4, 5], 3) == 2; _ledger.append(1)
assert bisect.bisect_right([1, 2, 3, 4, 5], 3) == 3; _ledger.append(1)
assert bisect.bisect_left([], 5) == 0; _ledger.append(1)
assert bisect.bisect_left([10, 20, 30], 25) == 2; _ledger.append(1)
assert bisect.bisect_left([1, 2, 2, 2, 3], 2) == 1; _ledger.append(1)
assert bisect.bisect_right([1, 2, 2, 2, 3], 2) == 4; _ledger.append(1)

# 3) bisect — insort/insort_left in-place insertion
_lst1 = [1, 3, 5]
bisect.insort(_lst1, 4)
assert _lst1 == [1, 3, 4, 5]; _ledger.append(1)
_lst2 = [1, 2, 2, 3]
bisect.insort_left(_lst2, 2)
assert _lst2 == [1, 2, 2, 2, 3]; _ledger.append(1)

# 4) heapq — hasattr surface
assert hasattr(heapq, "heappush") == True; _ledger.append(1)
assert hasattr(heapq, "heappop") == True; _ledger.append(1)
assert hasattr(heapq, "heapify") == True; _ledger.append(1)
assert hasattr(heapq, "heapreplace") == True; _ledger.append(1)
assert hasattr(heapq, "heappushpop") == True; _ledger.append(1)
assert hasattr(heapq, "nlargest") == True; _ledger.append(1)
assert hasattr(heapq, "nsmallest") == True; _ledger.append(1)
assert hasattr(heapq, "merge") == True; _ledger.append(1)

# 5) heapq — heapify/heappush/heappop value contracts
_h1 = [3, 1, 4, 1, 5, 9, 2, 6]
heapq.heapify(_h1)
assert _h1[0] == 1; _ledger.append(1)
_h2: list[int] = []
heapq.heappush(_h2, 5)
heapq.heappush(_h2, 2)
heapq.heappush(_h2, 8)
assert _h2[0] == 2; _ledger.append(1)
_h3 = [1, 2, 3, 4, 5]
heapq.heapify(_h3)
assert heapq.heappop(_h3) == 1; _ledger.append(1)

# 6) heapq — nlargest/nsmallest
assert heapq.nlargest(3, [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]) == [10, 9, 8]; _ledger.append(1)
assert heapq.nsmallest(3, [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]) == [1, 2, 3]; _ledger.append(1)

# 7) array — hasattr surface + typecode membership
assert hasattr(array, "array") == True; _ledger.append(1)
assert hasattr(array, "typecodes") == True; _ledger.append(1)
assert hasattr(array, "ArrayType") == True; _ledger.append(1)
assert ("i" in array.typecodes) == True; _ledger.append(1)

# 8) struct — hasattr surface
assert hasattr(struct, "pack") == True; _ledger.append(1)
assert hasattr(struct, "unpack") == True; _ledger.append(1)
assert hasattr(struct, "calcsize") == True; _ledger.append(1)
assert hasattr(struct, "pack_into") == True; _ledger.append(1)
assert hasattr(struct, "unpack_from") == True; _ledger.append(1)
assert hasattr(struct, "Struct") == True; _ledger.append(1)
assert hasattr(struct, "error") == True; _ledger.append(1)
assert hasattr(struct, "iter_unpack") == True; _ledger.append(1)

# 9) struct — calcsize for fixed integer/double formats
assert struct.calcsize("i") == 4; _ledger.append(1)
assert struct.calcsize("d") == 8; _ledger.append(1)
assert struct.calcsize("ii") == 8; _ledger.append(1)
assert struct.calcsize("4i") == 16; _ledger.append(1)

# 10) struct — int pack/unpack roundtrips
assert struct.pack("i", 5) == b"\x05\x00\x00\x00"; _ledger.append(1)
assert struct.unpack("i", struct.pack("i", 5)) == (5,); _ledger.append(1)
assert struct.unpack("ii", struct.pack("ii", 1, 2)) == (1, 2); _ledger.append(1)
assert struct.unpack("q", struct.pack("q", 100))[0] == 100; _ledger.append(1)
assert struct.unpack("B", struct.pack("B", 255))[0] == 255; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_bisect_heapq_array_struct_value_ops {sum(_ledger)} asserts")
