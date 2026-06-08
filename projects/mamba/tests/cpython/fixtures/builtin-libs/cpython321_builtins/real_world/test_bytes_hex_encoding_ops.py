# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "cpython321_builtins"
# dimension = "real_world"
# case = "test_bytes_hex_encoding_ops"
# subject = "cpython321.test_bytes_hex_encoding_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_bytes_hex_encoding_ops.py"
# status = "filled"
# ///
"""cpython321.test_bytes_hex_encoding_ops: execute CPython 3.12 seed test_bytes_hex_encoding_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for bytes hex/encoding surfaces
# beyond test_bytes_ops (which covers index/slice/len/bytes(list)/
# bytes(int)).
# Surface: bytes(str, encoding) constructor; .decode() with implicit
# and explicit utf-8; .hex() of bytes; bytes.fromhex() class method;
# concatenation via `+`; multiplication via `*`; bytes equality.
_ledger: list[int] = []

# bytes(str, encoding) encodes the string to bytes
assert bytes("abc", "utf-8") == b"abc"; _ledger.append(1)
assert bytes("hi", "utf-8") == b"hi"; _ledger.append(1)

# .decode() with no argument defaults to utf-8
assert b"hello".decode() == "hello"; _ledger.append(1)
# .decode("utf-8") is the explicit form of the same
assert b"hello".decode("utf-8") == "hello"; _ledger.append(1)

# .hex() returns the lowercase hex representation as a str
assert b"abc".hex() == "616263"; _ledger.append(1)
assert b"\x00\xff".hex() == "00ff"; _ledger.append(1)

# bytes.fromhex inverts .hex()
assert bytes.fromhex("616263") == b"abc"; _ledger.append(1)
# bytes.fromhex tolerates an empty string
assert bytes.fromhex("") == b""; _ledger.append(1)
# bytes.fromhex round-trip
assert bytes.fromhex(b"abc".hex()) == b"abc"; _ledger.append(1)

# Concatenation via `+` yields a new bytes object
assert b"abc" + b"def" == b"abcdef"; _ledger.append(1)

# Multiplication via `*` repeats the bytes n times
assert b"ab" * 3 == b"ababab"; _ledger.append(1)
# Multiplication by zero yields empty bytes
assert b"ab" * 0 == b""; _ledger.append(1)

# Bytes equality on identical content
assert b"abc" == b"abc"; _ledger.append(1)
# Bytes inequality on different content
assert b"abc" != b"def"; _ledger.append(1)

# bytes(int) creates a zero-padded buffer of that length
assert bytes(3) == b"\x00\x00\x00"; _ledger.append(1)

# bytes(list_of_ints) interprets each int as a byte value
assert bytes([72, 105]) == b"Hi"; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: test_bytes_hex_encoding_ops {sum(_ledger)} asserts")
