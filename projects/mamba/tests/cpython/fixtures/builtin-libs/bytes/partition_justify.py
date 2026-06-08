# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""bytes partition/rpartition + center/ljust/rjust justification."""

# partition splits on the first match into a 3-tuple.
assert b"mississippi".partition(b"ss") == (b"mi", b"ss", b"issippi")
# A missing separator puts the whole string in the head, empties after.
assert b"mississippi".partition(b"w") == (b"mississippi", b"", b"")
# rpartition splits on the LAST match; misses fill the tail instead.
assert b"mississippi".rpartition(b"ss") == (b"missi", b"ss", b"ippi")
assert b"mississippi".rpartition(b"w") == (b"", b"", b"mississippi")

# center/ljust/rjust pad to a width; the default fill is a space.
assert b"abc".center(7) == b"  abc  "
assert b"abc".ljust(7) == b"abc    "
assert b"abc".rjust(7) == b"    abc"
# A custom single-byte fill works, supplied as bytes or bytearray.
assert b"abc".center(7, b"-") == b"--abc--"
assert b"abc".ljust(7, b"*") == b"abc****"
assert b"abc".rjust(7, bytearray(b"+")) == b"++++abc"
# Width <= len returns the original content unchanged.
assert b"abc".center(2) == b"abc"
assert b"abc".ljust(0) == b"abc"

# Odd padding favors the left side in center (CPython rule).
assert b"ab".center(5, b".") == b"..ab."

# bytearray justification returns a bytearray.
res = bytearray(b"x").center(3, b"-")
assert res == b"-x-"
assert type(res) is bytearray

# A multi-byte fill is a ValueError.
try:
    b"abc".center(7, b"--")
    raise AssertionError("expected error")
except (TypeError, ValueError):
    pass

print("partition_justify OK")
