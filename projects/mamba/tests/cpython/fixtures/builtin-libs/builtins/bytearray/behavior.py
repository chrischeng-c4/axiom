"""Behavior contract for builtins.bytearray.

Each block tests one semantic rule. Expected values are what CPython
3.12 produces. mamba PASS = matches CPython exactly.

# type-regime: monomorphic
"""

# Rule 1: bytearray() with no args returns empty bytearray
ba = bytearray()
assert len(ba) == 0, f"bytearray() len = {len(ba)!r}"

# Rule 2: bytearray(int) — zero-filled
ba = bytearray(3)
assert ba == bytearray(b"\x00\x00\x00"), f"bytearray(3) = {bytes(ba)!r}"

# Rule 3: bytearray(bytes)
ba = bytearray(b"hello")
assert len(ba) == 5, f"len = {len(ba)!r}"
assert ba[0] == 104, f"ba[0] = {ba[0]!r}"  # ord('h')

# Rule 4: bytearray(iterable of ints)
ba = bytearray([65, 66, 67])
assert bytes(ba) == b"ABC", f"bytearray([65,66,67]) = {bytes(ba)!r}"

# Rule 5: mutable — item assignment
ba = bytearray(b"hello")
ba[0] = 72  # 'H'
assert ba[0] == 72, f"ba[0] after set = {ba[0]!r}"
assert bytes(ba) == b"Hello", f"after set: {bytes(ba)!r}"

# Rule 6: mutable — slice assignment
ba = bytearray(b"hello")
ba[1:3] = b"EL"
assert bytes(ba) == b"hELlo", f"slice assign = {bytes(ba)!r}"

# Rule 7: append / extend
ba = bytearray(b"ab")
ba.append(99)  # 'c'
assert bytes(ba) == b"abc", f"append = {bytes(ba)!r}"
ba.extend(b"de")
assert bytes(ba) == b"abcde", f"extend = {bytes(ba)!r}"

# Rule 8: pop
ba = bytearray(b"abc")
v = ba.pop()
assert v == 99, f"pop = {v!r}"  # ord('c')
assert bytes(ba) == b"ab", f"after pop = {bytes(ba)!r}"

# Rule 9: decode
ba = bytearray(b"hello")
assert ba.decode("utf-8") == "hello", f"decode = {ba.decode('utf-8')!r}"

# Rule 10: hex / fromhex
ba = bytearray(b"\xff\x00")
assert ba.hex() == "ff00", f"hex = {ba.hex()!r}"
assert bytearray.fromhex("ff00") == bytearray(b"\xff\x00"), "fromhex"

# Rule 11: bytes(bytearray) converts to bytes
ba = bytearray(b"xyz")
b = bytes(ba)
assert b == b"xyz", f"bytes(bytearray) = {b!r}"
assert type(b) is bytes, f"type = {type(b).__name__!r}"

# Rule 12: bytearray is NOT hashable
_raised = False
try:
    hash(bytearray(b"x"))
except TypeError:
    _raised = True
assert _raised, "hash(bytearray) did not raise TypeError"

print("behavior OK")
