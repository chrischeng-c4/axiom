# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""builtin-libs/bytes: behavior asserts (CPython 3.12 oracle)."""

# Comparison is lexicographic over the byte values.
b1 = bytes([1, 2, 3])
b2 = bytes([1, 2, 3])
b3 = bytes([1, 3])
assert b1 == b2 and b1 != b3
assert b1 <= b2 and b1 < b3 and b1 <= b3
assert b1 >= b2 and b3 > b2 and b3 >= b2
assert not (b1 > b3) and not (b1 >= b3)
# bytes and bytearray with equal contents compare equal across types.
assert b"abc" == bytearray(b"abc")
assert bytearray(b"abc") < bytearray(b"abd")

# hex() round-trips through fromhex().
assert b"\x1a+0".hex() == "1a2b30"
assert bytes.fromhex("1a2b30") == b"\x1a+0"
# hex() accepts a separator and an optional bytes-per-group count.
three = bytes(b"\xb9\x01\xef")
assert three.hex(":") == "b9:01:ef"
assert three.hex(":", 2) == "b9:01ef"     # group right-to-left, 2 bytes each
assert three.hex("*", -2) == "b901*ef"    # negative groups left-to-right
assert three.hex(":", 0) == "b901ef"      # group size 0 == no separator

# An empty or multi-char separator is a ValueError.
for bad in ("", "xx"):
    try:
        three.hex(bad)
        raise AssertionError("expected ValueError")
    except ValueError:
        pass

# A few core sequence behaviors carry over.
assert b"abc" + b"def" == b"abcdef"
assert b"ab" * 3 == b"ababab"
assert len(b"abc") == 3

print("behavior OK")
