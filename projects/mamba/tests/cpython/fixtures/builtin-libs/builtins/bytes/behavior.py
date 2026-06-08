"""Behavior contract for builtins.bytes.

Each block tests one semantic rule. Expected values are what CPython
3.12 produces. mamba PASS = matches CPython exactly.

# type-regime: monomorphic
"""

# Rule 1: bytes() with no args returns b''
assert bytes() == b"", f"bytes() = {bytes()!r}"

# Rule 2: bytes(int) — zero-filled
b = bytes(3)
assert b == b"\x00\x00\x00", f"bytes(3) = {b!r}"
assert len(b) == 3, f"len(bytes(3)) = {len(b)!r}"

# Rule 3: bytes(iterable of ints)
b = bytes([65, 66, 67])
assert b == b"ABC", f"bytes([65,66,67]) = {b!r}"

# Rule 4: bytes literal / str.encode
b = b"hello"
assert len(b) == 5, f"len(b'hello') = {len(b)!r}"
assert b[0] == 104, f"b'hello'[0] = {b[0]!r}"  # ord('h') == 104

# Rule 5: indexing returns int
b = b"ABC"
assert b[0] == 65, f"b[0] = {b[0]!r}"
assert type(b[0]) is int, f"type(b[0]) = {type(b[0]).__name__!r}"

# Rule 6: slicing returns bytes
b = b"hello"
assert b[1:3] == b"el", f"b[1:3] = {b[1:3]!r}"
assert type(b[1:3]) is bytes, f"slice type = {type(b[1:3]).__name__!r}"

# Rule 7: bytes is immutable
_raised = False
try:
    b = b"hello"
    b[0] = 72  # type: ignore[index]
except TypeError:
    _raised = True
assert _raised, "bytes[0] = ... did not raise TypeError"

# Rule 8: decode
assert b"hello".decode("utf-8") == "hello", f"decode = {b'hello'.decode('utf-8')!r}"
assert b"caf\xc3\xa9".decode("utf-8") == "café", "decode utf-8 multibyte"

# Rule 9: hex
assert b"\xff\x00".hex() == "ff00", f"hex = {b'\\xff\\x00'.hex()!r}"
assert b"\xde\xad".hex() == "dead", f"hex = {b'\\xde\\xad'.hex()!r}"

# Rule 10: fromhex
assert bytes.fromhex("ff00") == b"\xff\x00", f"fromhex = {bytes.fromhex('ff00')!r}"

# Rule 11: find
b = b"hello world"
assert b.find(b"world") == 6, f"find = {b.find(b'world')!r}"
assert b.find(b"xyz") == -1, f"find miss = {b.find(b'xyz')!r}"

# Rule 12: replace
b = b"hello"
assert b.replace(b"l", b"r") == b"herro", f"replace = {b.replace(b'l', b'r')!r}"

# Rule 13: split
b = b"a,b,c"
assert b.split(b",") == [b"a", b"b", b"c"], f"split = {b.split(b',')!r}"

# Rule 14: startswith / endswith
assert b"hello".startswith(b"he"), "startswith failed"
assert b"hello".endswith(b"lo"), "endswith failed"

# Rule 15: count
assert b"hello".count(b"l") == 2, f"count = {b'hello'.count(b'l')!r}"

# Rule 16: in operator
assert b"ell" in b"hello", "b'ell' in b'hello' failed"

print("behavior OK")
