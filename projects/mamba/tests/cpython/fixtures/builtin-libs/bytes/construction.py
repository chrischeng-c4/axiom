# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""bytes/bytearray construction from the many accepted source forms."""

# bytes(int) / bytearray(int) build a zero-filled buffer of that length.
assert bytes(0) == b""
assert bytes(5) == b"\x00\x00\x00\x00\x00"
assert bytearray(3) == bytearray([0, 0, 0])

# From a list / tuple of ints.
assert bytes([65, 66, 67]) == b"ABC"
assert bytes((97, 98)) == b"ab"

# From a range and from an iterator/generator of ints.
assert bytes(range(5)) == b"\x00\x01\x02\x03\x04"
assert bytes(iter([1, 2, 3])) == b"\x01\x02\x03"
assert bytes(i for i in range(6) if i % 2) == b"\x01\x03\x05"

# From a set (membership preserved; order unspecified, so check via set).
assert set(bytes({43, 45})) == {43, 45}

# From an object exposing __getitem__ (old-style sequence protocol).
class Seq:
    def __getitem__(self, i):
        return (1, 2, 3)[i]
assert bytes(Seq()) == b"\x01\x02\x03"

# From a str + encoding.
assert bytes("café", "utf-8") == b"caf\xc3\xa9"
assert bytes("hi", "ascii") == b"hi"

# bytearray accepts the same forms and is mutable afterward.
ba = bytearray(range(4))
assert ba == bytearray([0, 1, 2, 3])

# Out-of-range or negative element values raise ValueError.
for bad in ([-1], [256], [257], [-10 ** 100], [10 ** 100]):
    try:
        bytes(bad)
        raise AssertionError("expected ValueError for %r" % bad)
    except ValueError:
        pass

# A bare str with no encoding is a TypeError.
try:
    bytes("oops")
    raise AssertionError("expected TypeError")
except TypeError:
    pass

print("construction OK")
