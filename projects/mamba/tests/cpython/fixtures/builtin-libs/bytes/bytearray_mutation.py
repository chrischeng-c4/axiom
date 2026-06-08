# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""bytearray in-place mutation: the mutable side of the bytes API."""

# append / extend / insert / remove / pop grow and shrink in place.
b = bytearray(b"abc")
b.append(ord("d"))
assert b == b"abcd"
b.extend(b"ef")
assert b == b"abcdef"
b.extend(range(2))          # extend accepts any int iterable
assert b == b"abcdef\x00\x01"
b.insert(0, ord("X"))
assert b == b"Xabcdef\x00\x01"
b.remove(ord("a"))          # removes first matching byte
assert b == b"Xbcdef\x00\x01"
assert b.pop() == 1         # pop() returns the int byte
assert b.pop(0) == ord("X")
assert b.pop(-1) == 0

# reverse / clear / copy.
r = bytearray(b"hello")
assert r.reverse() is None  # mutates in place, returns None
assert r == b"olleh"
r.clear()
assert r == b""
src = bytearray(b"abc")
dup = src.copy()
dup.append(ord("d"))
assert src == b"abc" and dup == b"abcd"   # copy is independent

# += (iconcat) mutates in place and preserves identity; *= repeats.
acc = bytearray(b"ab")
same = acc
acc += b"cd"
assert acc == b"abcd" and acc is same
acc *= 2
assert acc == b"abcdabcd" and acc is same

# Slice assignment can grow, shrink, or replace; del removes a region.
s = bytearray(range(10))
s[0:5] = bytearray([1, 1, 1])      # shrink
assert s == bytearray([1, 1, 1, 5, 6, 7, 8, 9])
s[3:3] = b"foo"                    # insert at a gap
assert s == bytearray([1, 1, 1, 102, 111, 111, 5, 6, 7, 8, 9])
del s[3:6]
assert s == bytearray([1, 1, 1, 5, 6, 7, 8, 9])

# Extended (stepped) slice assignment and deletion.
e = bytearray(range(10))
e[::2] = b"\x00\x00\x00\x00\x00"
assert e == bytearray([0, 1, 0, 3, 0, 5, 0, 7, 0, 9])
del e[::2]
assert e == bytearray([1, 3, 5, 7, 9])

# Single-element setitem takes an int in range(256).
g = bytearray(b"abc")
g[1] = 0
assert g == bytearray([97, 0, 99])

print("bytearray_mutation OK")
