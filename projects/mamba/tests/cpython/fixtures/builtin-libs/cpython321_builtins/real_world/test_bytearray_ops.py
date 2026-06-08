# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "cpython321_builtins"
# dimension = "real_world"
# case = "test_bytearray_ops"
# subject = "cpython321.test_bytearray_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_bytearray_ops.py"
# status = "filled"
# ///
"""cpython321.test_bytearray_ops: execute CPython 3.12 seed test_bytearray_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for the bytearray builtin type.
# Surface: bytearray(bytes) constructor preserving content; len();
# .append(int) extends by one byte; .extend(bytes) appends a span;
# in-place index assignment ba[i] = v mutates the buffer; reading
# ba[i] returns an int; slicing returns a new bytearray;
# bytearray(int) zero-pads to that length; bytearray(list_of_ints)
# from byte values; bytearray.fromhex inverts hex; .pop removes and
# returns the last byte; bytes(bytearray) round-trips the content.
_ledger: list[int] = []

# Constructor preserves bytes content
ba = bytearray(b"hello")
assert bytes(ba) == b"hello"; _ledger.append(1)
assert len(ba) == 5; _ledger.append(1)

# .append takes a single int byte value
ba.append(ord("!"))
assert bytes(ba) == b"hello!"; _ledger.append(1)
assert len(ba) == 6; _ledger.append(1)

# .extend appends every byte of the argument
ba.extend(b"abc")
assert bytes(ba) == b"hello!abc"; _ledger.append(1)

# In-place index assignment mutates a single byte
ba[0] = ord("H")
assert bytes(ba) == b"Hello!abc"; _ledger.append(1)
# Reading ba[i] returns the int byte value, not a one-byte bytes object
assert ba[0] == 72; _ledger.append(1)

# Slicing returns a new bytearray containing the slice
sl = ba[1:5]
assert bytes(sl) == b"ello"; _ledger.append(1)

# bytearray(int n) zero-pads to length n
ba2 = bytearray(5)
assert bytes(ba2) == b"\x00\x00\x00\x00\x00"; _ledger.append(1)
assert len(ba2) == 5; _ledger.append(1)

# bytearray(list_of_ints) interprets each int as a byte value
ba3 = bytearray([65, 66, 67])
assert bytes(ba3) == b"ABC"; _ledger.append(1)

# bytearray.fromhex parses a hex string into a bytearray
ba4 = bytearray.fromhex("616263")
assert bytes(ba4) == b"abc"; _ledger.append(1)

# .pop without an argument removes and returns the last byte
ba5 = bytearray(b"abcd")
v = ba5.pop()
assert v == ord("d"); _ledger.append(1)
assert bytes(ba5) == b"abc"; _ledger.append(1)

# bytes(bytearray) round-trips back to bytes preserving content
assert bytes(bytearray(b"round-trip")) == b"round-trip"; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: test_bytearray_ops {sum(_ledger)} asserts")
