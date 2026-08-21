# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""bytes repr/str/format and the integer view of bytes elements."""

# repr/str of bytes uses the b'...' literal form with hex escapes.
assert repr(b"") == "b''"
assert repr(b"abc") == "b'abc'"
assert str(b"\x80") == "b'\\x80'"
assert repr(bytes([0, 1, 254, 255])) == "b'\\x00\\x01\\xfe\\xff'"
# Quote selection mirrors CPython: prefers ', switches to " when needed.
assert repr(b"'") == 'b"\'"'
assert repr(b"'\"") == "b'\\'\"'"

# bytearray repr wraps in bytearray(b'...').
assert repr(bytearray(b"")) == "bytearray(b'')"
assert str(bytearray(b"\x80")) == "bytearray(b'\\x80')"
assert repr(bytearray([0, 1, 254, 255])) == "bytearray(b'\\x00\\x01\\xfe\\xff')"

# format(b, '') matches str(b); a non-empty spec is a TypeError.
for b in (b"abc", bytearray(b"abc")):
    assert format(b) == str(b)
    assert format(b, "") == str(b)
    try:
        format(b, "s")
        raise AssertionError("expected TypeError")
    except TypeError:
        pass

# Indexing a bytes yields an int; slicing yields bytes.
b = bytearray(b"\x00A\x7f\x80\xff")
assert b[0] == 0 and b[1] == 65 and b[-1] == 255
assert [ord(b[i:i + 1]) for i in range(len(b))] == [0, 65, 127, 128, 255]

# Iterating yields ints; list()/tuple() materialize them.
assert list(b"ABC") == [65, 66, 67]
assert tuple(b"\x01\x02") == (1, 2)
# reversed() walks the ints back to front.
assert list(reversed(b"ABC")) == [67, 66, 65]

# count/contains accept an int (a single byte value).
assert b"mississippi".count(ord("i")) == 4
assert ord("a") in b"abc"
assert 200 not in b"abc"
# An int outside range(256) is a ValueError in membership tests.
try:
    300 in b"abc"
    raise AssertionError("expected ValueError")
except ValueError:
    pass

print("repr_and_int_view OK")
