# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""builtins: bytearray join / extend / translate behavior and errors."""

# Distilled from CPython Lib/test/test_builtin.py bytearray cases
# (re-curated: dropped the misbehaving-iterator BufferError, an
# allocation-internal detail).

# join() consumes an iterator of bytes-like chunks.
sep = bytearray(b",")


def chunks():
    yield b"A"
    yield b"B"
    yield b"C"


assert sep.join(chunks()) == bytearray(b"A,B,C")

# join() also accepts a list of bytes objects.
assert bytearray(b"-").join([b"x", b"y"]) == bytearray(b"x-y")

# extend() over an iterable of ints appends each byte.
buf = bytearray(b"ab")
buf.extend([99, 100])  # c, d
assert buf == bytearray(b"abcd")

# extend() of an iterable yielding a non-int raises ValueError.
try:
    bytearray().extend(map(int, "X"))  # int('X') -> ValueError mid-iter
    raise AssertionError("expected ValueError")
except ValueError:
    pass

# translate() with a delete set removes those bytes.
assert bytearray(b"abcabc").translate(None, b"b") == bytearray(b"acac")

# translate() with a 1:1 table maps bytes.
table = bytes.maketrans(b"abc", b"xyz")
assert bytearray(b"cab").translate(table) == bytearray(b"zxy")

# A translation table that is not 256 bytes raises ValueError.
try:
    bytearray(b"abc").translate(b"1", b"x")
    raise AssertionError("expected ValueError")
except ValueError:
    pass

# A non-bytes delete argument raises TypeError.
try:
    bytearray(b"abc").translate(b"1" * 256, 1)
    raise AssertionError("expected TypeError")
except TypeError:
    pass

print("bytearray_methods OK")
