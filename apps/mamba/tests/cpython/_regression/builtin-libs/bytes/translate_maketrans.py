# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""bytes.maketrans + translate: byte remapping and deletion."""

# maketrans builds a 256-entry table mapping the `frm` bytes to `to`.
table = bytes.maketrans(b"abc", b"xyz")
assert len(table) == 256
assert table[ord("a")] == ord("x")
assert table[ord("b")] == ord("y")
assert table[ord("c")] == ord("z")
assert table[ord("d")] == ord("d")          # untouched byte maps to itself

# translate applies the table.
assert b"abcabc".translate(table) == b"xyzxyz"
# translate(None, ...) keeps every byte but honors the delete set.
assert b"hello".translate(None, b"l") == b"heo"
# A table plus a delete set: delete happens, then remap survives.
rosetta = bytearray(range(256))
rosetta[ord("o")] = ord("e")
b = b"hello"
assert b.translate(bytes(rosetta), b"l") == b"hee"   # 'l' deleted, 'o'->'e'
assert b == b"hello"                                  # source unchanged
# delete= keyword form.
assert b.translate(None, delete=b"e") == b"hllo"

# maketrans raises ValueError when the two arguments differ in length.
try:
    bytes.maketrans(b"abc", b"xyzq")
    raise AssertionError("expected ValueError")
except ValueError:
    pass

# maketrans rejects str arguments (must be bytes-like).
try:
    bytes.maketrans("abc", "def")
    raise AssertionError("expected TypeError")
except TypeError:
    pass

# bytearray.translate returns a bytearray.
res = bytearray(b"abc").translate(table)
assert res == b"xyz"
assert type(res) is bytearray

print("translate_maketrans OK")
