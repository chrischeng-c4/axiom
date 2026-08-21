# Operational AssertionPass seed for the matching `array.array` surface
# (typed-storage construction + tolist + mutators + read-only typecode
# / itemsize properties). There is no existing array seed.
#
# `array.array` is a fixed-typecode dense numeric buffer — the
# constructor accepts a typecode plus an initializer (list of numbers
# OR bytes for byte typecodes), and the public methods (.append,
# .extend, .insert, .pop, .remove, .reverse, .count, .index, .tolist,
# .tobytes, .frombytes) mirror the list API but operate on a single
# numeric width.
#
# This fixture exercises the matching subset between mamba and CPython:
#   • All standard typecodes (b, B, h, H, i, I, l, L, q, Q, f, d) with
#     a list initializer round-trip through .tolist();
#   • Byte-typecode (b/B) construction from a bytes object;
#   • Mutators (.append, .extend, .insert, .pop default, .pop(index),
#     .remove, .reverse) — verified post-mutation via .tolist();
#   • Read-only properties: .typecode (single char) and .itemsize
#     (bytes per element, 64-bit Darwin/arm64 default: l/q/d=8, i/f=4,
#     h=2, b/B=1);
#   • .count / .index — list-like lookup;
#   • Round-trip through .tobytes / .frombytes for byte typecodes.
#
# Behavioral edges that DIVERGE on mamba (len()/== compare/[i] index/
# iteration via iter()/typecode-validation/overflow-checking) are
# covered in `lang_array_index_eq_iter_silent.py`.
import array

_ledger: list[int] = []

# 1) Construction with each typecode + tolist round-trip
assert array.array("b", [1, 2, 3]).tolist() == [1, 2, 3]; _ledger.append(1)
assert array.array("B", [1, 2, 3]).tolist() == [1, 2, 3]; _ledger.append(1)
assert array.array("h", [1, 2, 3]).tolist() == [1, 2, 3]; _ledger.append(1)
assert array.array("H", [1, 2, 3]).tolist() == [1, 2, 3]; _ledger.append(1)
assert array.array("i", [10, 20, 30]).tolist() == [10, 20, 30]; _ledger.append(1)
assert array.array("I", [10, 20, 30]).tolist() == [10, 20, 30]; _ledger.append(1)
assert array.array("l", [100, 200, 300]).tolist() == [100, 200, 300]; _ledger.append(1)
assert array.array("L", [100, 200, 300]).tolist() == [100, 200, 300]; _ledger.append(1)
assert array.array("q", [1, 2, 3]).tolist() == [1, 2, 3]; _ledger.append(1)
assert array.array("Q", [1, 2, 3]).tolist() == [1, 2, 3]; _ledger.append(1)
assert array.array("f", [1.5, 2.5, 3.5]).tolist() == [1.5, 2.5, 3.5]; _ledger.append(1)
assert array.array("d", [1.5, 2.5, 3.5]).tolist() == [1.5, 2.5, 3.5]; _ledger.append(1)

# 2) Byte typecodes accept a bytes initializer
assert array.array("B", b"abc").tolist() == [97, 98, 99]; _ledger.append(1)
assert array.array("b", b"ABC").tolist() == [65, 66, 67]; _ledger.append(1)
# Bytes round-trip via tobytes/frombytes
assert array.array("b", [65, 66, 67]).tobytes() == b"ABC"; _ledger.append(1)
assert array.array("B", [65, 66, 67]).tobytes() == b"ABC"; _ledger.append(1)

_fb_b = array.array("b")
_fb_b.frombytes(b"\x01\x02\x03")
assert _fb_b.tolist() == [1, 2, 3]; _ledger.append(1)

_fb_B = array.array("B")
_fb_B.frombytes(b"\x01\x02\x03")
assert _fb_B.tolist() == [1, 2, 3]; _ledger.append(1)

# 3) Mutators — verify via .tolist after each operation
_app = array.array("i", [1, 2])
_app.append(99)
assert _app.tolist() == [1, 2, 99]; _ledger.append(1)

_ext = array.array("i", [1, 2])
_ext.extend([10, 20])
assert _ext.tolist() == [1, 2, 10, 20]; _ledger.append(1)

_ins = array.array("i", [1, 2, 3])
_ins.insert(1, 99)
assert _ins.tolist() == [1, 99, 2, 3]; _ledger.append(1)

_ins2 = array.array("i", [10, 20])
_ins2.insert(0, 5)
assert _ins2.tolist() == [5, 10, 20]; _ledger.append(1)

_pop = array.array("i", [1, 2, 3])
_popped = _pop.pop()
assert _popped == 3; _ledger.append(1)
assert _pop.tolist() == [1, 2]; _ledger.append(1)

_pop2 = array.array("i", [1, 2, 3])
_popped2 = _pop2.pop(0)
assert _popped2 == 1; _ledger.append(1)
assert _pop2.tolist() == [2, 3]; _ledger.append(1)

_rem = array.array("i", [1, 2, 3, 2])
_rem.remove(2)
# remove drops the FIRST occurrence
assert _rem.tolist() == [1, 3, 2]; _ledger.append(1)

_rev = array.array("i", [1, 2, 3])
_rev.reverse()
assert _rev.tolist() == [3, 2, 1]; _ledger.append(1)

_rev2 = array.array("i", [1])
_rev2.reverse()
assert _rev2.tolist() == [1]; _ledger.append(1)

# 4) Read-only properties — typecode (single char) and itemsize (bytes per item)
assert array.array("i", [1, 2]).typecode == "i"; _ledger.append(1)
assert array.array("b", [1, 2]).typecode == "b"; _ledger.append(1)
assert array.array("B", [1, 2]).typecode == "B"; _ledger.append(1)
assert array.array("h", [1, 2]).typecode == "h"; _ledger.append(1)
assert array.array("H", [1, 2]).typecode == "H"; _ledger.append(1)
assert array.array("l", [1, 2]).typecode == "l"; _ledger.append(1)
assert array.array("L", [1, 2]).typecode == "L"; _ledger.append(1)
# q/Q typecode-identity is not part of the matching subset — mamba
# normalizes "q"/"Q" to "l"/"L". The 64-bit storage width is still
# correct, so itemsize stays in the matching subset below.
assert array.array("f", [1.0]).typecode == "f"; _ledger.append(1)
assert array.array("d", [1.0]).typecode == "d"; _ledger.append(1)

# itemsize — fixed widths from the typecode (Darwin/arm64 64-bit):
#   b/B = 1, h/H = 2, i/I = 4, l/L = 8, q/Q = 8, f = 4, d = 8
assert array.array("b", [1]).itemsize == 1; _ledger.append(1)
assert array.array("B", [1]).itemsize == 1; _ledger.append(1)
assert array.array("h", [1]).itemsize == 2; _ledger.append(1)
assert array.array("H", [1]).itemsize == 2; _ledger.append(1)
assert array.array("i", [1]).itemsize == 4; _ledger.append(1)
assert array.array("I", [1]).itemsize == 4; _ledger.append(1)
assert array.array("l", [1]).itemsize == 8; _ledger.append(1)
assert array.array("L", [1]).itemsize == 8; _ledger.append(1)
assert array.array("q", [1]).itemsize == 8; _ledger.append(1)
assert array.array("Q", [1]).itemsize == 8; _ledger.append(1)
assert array.array("f", [1.0]).itemsize == 4; _ledger.append(1)
assert array.array("d", [1.0]).itemsize == 8; _ledger.append(1)

# 5) count / index
assert array.array("i", [1, 2, 1, 3, 1]).count(1) == 3; _ledger.append(1)
assert array.array("i", [1, 2, 3]).count(99) == 0; _ledger.append(1)
assert array.array("i", [10, 20, 30, 40]).count(20) == 1; _ledger.append(1)
assert array.array("i", [1, 2, 3]).count(2) == 1; _ledger.append(1)

assert array.array("i", [10, 20, 30]).index(20) == 1; _ledger.append(1)
assert array.array("i", [10, 20, 30]).index(10) == 0; _ledger.append(1)
assert array.array("i", [10, 20, 30]).index(30) == 2; _ledger.append(1)
# index returns FIRST occurrence
assert array.array("i", [5, 7, 5, 7]).index(5) == 0; _ledger.append(1)
assert array.array("i", [5, 7, 5, 7]).index(7) == 1; _ledger.append(1)

# 6) Combined mutator pipeline — make sure state survives through chain
_pipe = array.array("i", [1, 2, 3])
_pipe.append(4)
_pipe.extend([5, 6])
_pipe.insert(0, 0)
assert _pipe.tolist() == [0, 1, 2, 3, 4, 5, 6]; _ledger.append(1)
assert _pipe.count(0) == 1; _ledger.append(1)
assert _pipe.index(6) == 6; _ledger.append(1)
_pipe.reverse()
assert _pipe.tolist() == [6, 5, 4, 3, 2, 1, 0]; _ledger.append(1)
_pipe.remove(3)
assert _pipe.tolist() == [6, 5, 4, 2, 1, 0]; _ledger.append(1)

# 7) Empty array — tolist returns empty list, typecode + itemsize still resolve
_empty = array.array("i")
assert _empty.tolist() == []; _ledger.append(1)
assert _empty.typecode == "i"; _ledger.append(1)
assert _empty.itemsize == 4; _ledger.append(1)
# Append into the empty buffer works
_empty.append(42)
assert _empty.tolist() == [42]; _ledger.append(1)

# Float typecode mutators
_fl = array.array("f", [1.5, 2.5])
_fl.append(3.5)
assert _fl.tolist() == [1.5, 2.5, 3.5]; _ledger.append(1)

# Negative-value tolist round trip on signed typecodes
assert array.array("b", [-128, -1, 0, 1, 127]).tolist() == [-128, -1, 0, 1, 127]; _ledger.append(1)
assert array.array("h", [-32768, 0, 32767]).tolist() == [-32768, 0, 32767]; _ledger.append(1)
assert array.array("i", [-2147483648, 0, 2147483647]).tolist() == [-2147483648, 0, 2147483647]; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_array_typed_storage_ops {sum(_ledger)} asserts")
