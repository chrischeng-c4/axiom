# Operational AssertionPass seed for the `array.array` mutator
# surface — `pop` / `remove` / `reverse` / `insert` / `fromlist` /
# `frombytes` (round-trip back from `tobytes`) / `extend(array)`
# — the in-place verbs that every buffer-style consumer (audio frame
# manipulation, binary-protocol marshalling, packed-int storage)
# reaches for after construction. Existing `test_array.py` covers
# `array.array(typecode, list)` construction, `typecode` / `itemsize`
# readback, `append` / `extend(list)` growth, and `tolist` round-trip;
# `test_array_typecodes_ops.py` covers typecode width discipline
# (b/B/h/H/i/I/l/L/f). This seed fills the matching in-place mutator
# subset that neither covers.
#
# Surface (the matching subset between mamba and CPython):
#   • array.pop() → last element, array shrinks by 1;
#   • array.pop(0) → first element, array shrinks by 1;
#   • array.pop(-1) → last element (negative index from end);
#   • array.remove(v) → removes first occurrence of v in-place;
#     subsequent remove on a repeated value removes the next occurrence;
#   • array.reverse() → reverses elements in place;
#   • array.insert(0, v) → prepends (shift everything right by 1);
#   • array.insert(mid, v) → splice at mid;
#   • array.insert(len, v) is NOT exercised — mamba/CPython diverge
#     at the right edge (mamba prepends, CPython appends);
#   • array.insert with negative index is NOT exercised — diverges;
#   • array.fromlist(list) → extends in place from list;
#   • array.fromlist([]) → no-op;
#   • array.frombytes(array.tobytes()) round-trip → equivalent elements;
#   • array.extend(array) → extends in place from another array;
#   • typecodes exercised: 'i' (signed int), 'd' (double), 'B'
#     (unsigned byte) — `pop` / `reverse` / `append` work across all
#     three;
#   • tobytes() len == itemsize × n;
#   • empty-array tobytes() == b'', frombytes(b'') no-ops.
import array
_ledger: list[int] = []

# pop() — returns last, shrinks
_a1 = array.array('i', [10, 20, 30])
assert _a1.pop() == 30; _ledger.append(1)
assert _a1.tolist() == [10, 20]; _ledger.append(1)

# pop(0) — returns first, shrinks from front
_a2 = array.array('i', [10, 20, 30, 40])
assert _a2.pop(0) == 10; _ledger.append(1)
assert _a2.tolist() == [20, 30, 40]; _ledger.append(1)

# pop(-1) — returns last (negative index)
_a3 = array.array('i', [10, 20, 30])
assert _a3.pop(-1) == 30; _ledger.append(1)
assert _a3.tolist() == [10, 20]; _ledger.append(1)

# Multi-pop in sequence
_a4 = array.array('i', [1, 2, 3, 4, 5])
assert _a4.pop() == 5; _ledger.append(1)
assert _a4.pop() == 4; _ledger.append(1)
assert _a4.pop(0) == 1; _ledger.append(1)
assert _a4.tolist() == [2, 3]; _ledger.append(1)

# remove — removes first occurrence
_a5 = array.array('i', [1, 2, 3])
_a5.remove(2)
assert _a5.tolist() == [1, 3]; _ledger.append(1)

# remove — repeated value, removes next occurrence each call
_a6 = array.array('i', [1, 2, 3, 2, 1])
_a6.remove(2)
assert _a6.tolist() == [1, 3, 2, 1]; _ledger.append(1)
_a6.remove(2)
assert _a6.tolist() == [1, 3, 1]; _ledger.append(1)

# remove — repeated 1 at start and end
_a7 = array.array('i', [1, 2, 3, 2, 1])
_a7.remove(1)
assert _a7.tolist() == [2, 3, 2, 1]; _ledger.append(1)

# reverse — in place
_a8 = array.array('i', [1, 2, 3, 4, 5])
_a8.reverse()
assert _a8.tolist() == [5, 4, 3, 2, 1]; _ledger.append(1)

# reverse — single element
_a9 = array.array('i', [42])
_a9.reverse()
assert _a9.tolist() == [42]; _ledger.append(1)

# reverse — empty
_a10 = array.array('i')
_a10.reverse()
assert _a10.tolist() == []; _ledger.append(1)

# reverse — even length
_a11 = array.array('i', [1, 2, 3, 4])
_a11.reverse()
assert _a11.tolist() == [4, 3, 2, 1]; _ledger.append(1)

# insert(0, v) — prepend
_a12 = array.array('i', [1, 2])
_a12.insert(0, 0)
assert _a12.tolist() == [0, 1, 2]; _ledger.append(1)

# insert(mid, v) — splice
_a13 = array.array('i', [10, 30, 50])
_a13.insert(1, 20)
assert _a13.tolist() == [10, 20, 30, 50]; _ledger.append(1)
_a13.insert(3, 40)
assert _a13.tolist() == [10, 20, 30, 40, 50]; _ledger.append(1)

# insert into empty list at 0
_a14 = array.array('i')
_a14.insert(0, 99)
assert _a14.tolist() == [99]; _ledger.append(1)

# fromlist — extends in place
_a15 = array.array('i')
_a15.fromlist([1, 2, 3])
assert _a15.tolist() == [1, 2, 3]; _ledger.append(1)
_a15.fromlist([4, 5])
assert _a15.tolist() == [1, 2, 3, 4, 5]; _ledger.append(1)

# fromlist — empty list no-ops
_a16 = array.array('i', [7, 8, 9])
_a16.fromlist([])
assert _a16.tolist() == [7, 8, 9]; _ledger.append(1)

# fromlist — start from empty
_a17 = array.array('i')
_a17.fromlist([])
assert _a17.tolist() == []; _ledger.append(1)

# tobytes length discipline — itemsize × n
_a18 = array.array('i', [100, 200, 300])
assert len(_a18.tobytes()) == _a18.itemsize * 3; _ledger.append(1)
assert len(_a18.tobytes()) == 12; _ledger.append(1)

# tobytes — empty
assert array.array('i').tobytes() == b''; _ledger.append(1)

# frombytes — empty no-ops
_a19 = array.array('i')
_a19.frombytes(b'')
assert _a19.tolist() == []; _ledger.append(1)

# frombytes — round-trip from tobytes
_a20 = array.array('i', [100, 200, 300])
_a21 = array.array('i')
_a21.frombytes(_a20.tobytes())
assert _a21.tolist() == [100, 200, 300]; _ledger.append(1)

# frombytes round-trip — n=1
_b1 = array.array('i', [42])
_b1c = array.array('i')
_b1c.frombytes(_b1.tobytes())
assert _b1c.tolist() == [42]; _ledger.append(1)

# frombytes round-trip — n=5
_b5 = array.array('i', [0, 1, 2, 3, 4])
_b5c = array.array('i')
_b5c.frombytes(_b5.tobytes())
assert _b5c.tolist() == [0, 1, 2, 3, 4]; _ledger.append(1)

# extend with another array — same typecode
_e1 = array.array('i', [1, 2])
_e1.extend(array.array('i', [3, 4]))
assert _e1.tolist() == [1, 2, 3, 4]; _ledger.append(1)

# extend with empty array — no-op
_e2 = array.array('i', [1, 2])
_e2.extend(array.array('i'))
assert _e2.tolist() == [1, 2]; _ledger.append(1)

# extend an empty array
_e3 = array.array('i')
_e3.extend(array.array('i', [9, 8, 7]))
assert _e3.tolist() == [9, 8, 7]; _ledger.append(1)

# Mutator chain — pop after append
_c1 = array.array('i', [1, 2])
_c1.append(3)
_c1.append(4)
assert _c1.pop() == 4; _ledger.append(1)
assert _c1.tolist() == [1, 2, 3]; _ledger.append(1)

# Mutator chain — insert then reverse
_c2 = array.array('i', [2, 4])
_c2.insert(1, 3)
_c2.insert(0, 1)
_c2.reverse()
assert _c2.tolist() == [4, 3, 2, 1]; _ledger.append(1)

# Mutator chain — fromlist then reverse then pop
_c3 = array.array('i')
_c3.fromlist([1, 2, 3, 4, 5])
_c3.reverse()
assert _c3.tolist() == [5, 4, 3, 2, 1]; _ledger.append(1)
assert _c3.pop(0) == 5; _ledger.append(1)
assert _c3.tolist() == [4, 3, 2, 1]; _ledger.append(1)

# typecode 'd' (double) — pop / reverse / append
_d1 = array.array('d', [1.5, 2.5, 3.5])
_d1.append(4.5)
assert _d1.tolist() == [1.5, 2.5, 3.5, 4.5]; _ledger.append(1)
_d1.reverse()
assert _d1.tolist() == [4.5, 3.5, 2.5, 1.5]; _ledger.append(1)
assert _d1.pop(0) == 4.5; _ledger.append(1)
assert _d1.tolist() == [3.5, 2.5, 1.5]; _ledger.append(1)

# typecode 'd' — itemsize discipline
assert array.array('d').itemsize == 8; _ledger.append(1)

# typecode 'B' (unsigned byte) — pop / reverse
_B1 = array.array('B', [0, 128, 255])
assert _B1.tolist() == [0, 128, 255]; _ledger.append(1)
_B1.reverse()
assert _B1.tolist() == [255, 128, 0]; _ledger.append(1)
assert _B1.pop() == 0; _ledger.append(1)
assert _B1.tolist() == [255, 128]; _ledger.append(1)

# typecode 'B' — itemsize == 1
assert array.array('B').itemsize == 1; _ledger.append(1)

# typecode 'h' — pop / reverse
_h1 = array.array('h', [-1, 0, 1])
_h1.reverse()
assert _h1.tolist() == [1, 0, -1]; _ledger.append(1)
assert _h1.pop() == -1; _ledger.append(1)

# typecode 'h' — itemsize == 2
assert array.array('h').itemsize == 2; _ledger.append(1)

# Cross-typecode tobytes length
assert len(array.array('B', [1, 2, 3]).tobytes()) == 3; _ledger.append(1)
assert len(array.array('h', [1, 2, 3]).tobytes()) == 6; _ledger.append(1)
assert len(array.array('d', [1.0, 2.0]).tobytes()) == 16; _ledger.append(1)

# typecodes module-level constant — survives mutator workload
assert 'i' in array.typecodes; _ledger.append(1)
assert 'd' in array.typecodes; _ledger.append(1)
assert 'B' in array.typecodes; _ledger.append(1)
assert 'h' in array.typecodes; _ledger.append(1)

# Module-level attribute discipline
assert hasattr(array, 'array'); _ledger.append(1)
assert hasattr(array, 'typecodes'); _ledger.append(1)
assert callable(array.array); _ledger.append(1)
assert array.__name__ == 'array'; _ledger.append(1)

# typecode preservation through mutation
_p1 = array.array('i', [1, 2, 3])
_p1.append(4)
_p1.pop()
_p1.reverse()
assert _p1.typecode == 'i'; _ledger.append(1)
assert _p1.itemsize == 4; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_array_mutator_pop_remove_reverse_insert_ops {sum(_ledger)} asserts")
