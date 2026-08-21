# Operational AssertionPass seed for bytes-literal operators and
# slicing — surface not deeply covered by `test_bytes_ops`,
# `test_bytes_hex_encoding_ops`, or `test_bytes_method_extras_ops`
# (which exercise upper/lower/split/replace/decode), or by
# `lang_string_prefixes.py` (which covers the b"abc"[i] indexing
# return-int invariant). This seed asserts the algebra of bytes:
# multiplication `b"a" * n` for n=0, n=1, n>1; concatenation `+`;
# membership `b"x" in b"...`; negative indexing; multi-step slicing
# (start:stop, ::step, ::-1 reverse); equality across distinct
# literal forms; len; b"".join() for empty and non-empty separators
# and lists; bytes constructor from list-of-int and from int (n
# zero bytes).
_ledger: list[int] = []

# Negative indexing — single-element returns int
assert b"abc"[-1] == 99; _ledger.append(1)
assert b"abc"[-2] == 98; _ledger.append(1)
assert b"abc"[-3] == 97; _ledger.append(1)

# Positive indexing — returns int
assert b"abcdef"[0] == 97; _ledger.append(1)
assert b"abcdef"[5] == 102; _ledger.append(1)

# Slicing — start:stop returns bytes
assert b"abcdef"[1:4] == b"bcd"; _ledger.append(1)
assert b"abcdef"[:3] == b"abc"; _ledger.append(1)
assert b"abcdef"[3:] == b"def"; _ledger.append(1)
assert b"abcdef"[:] == b"abcdef"; _ledger.append(1)
assert b"abcdef"[2:2] == b""; _ledger.append(1)

# Step slicing
assert b"abcdef"[::2] == b"ace"; _ledger.append(1)
assert b"abcdef"[1::2] == b"bdf"; _ledger.append(1)
assert b"abcdef"[::3] == b"ad"; _ledger.append(1)

# Reverse via [::-1]
assert b"abcdef"[::-1] == b"fedcba"; _ledger.append(1)
assert b"hello"[::-1] == b"olleh"; _ledger.append(1)
assert b""[::-1] == b""; _ledger.append(1)

# Slice with negative start/stop
assert b"abcdef"[-3:] == b"def"; _ledger.append(1)
assert b"abcdef"[:-2] == b"abcd"; _ledger.append(1)
assert b"abcdef"[-4:-1] == b"cde"; _ledger.append(1)

# Concatenation — bytes + bytes
assert b"abc" + b"def" == b"abcdef"; _ledger.append(1)
assert b"" + b"hello" == b"hello"; _ledger.append(1)
assert b"hello" + b"" == b"hello"; _ledger.append(1)

# Multiplication — bytes * n
assert b"a" * 3 == b"aaa"; _ledger.append(1)
assert b"ab" * 4 == b"abababab"; _ledger.append(1)
assert b"" * 5 == b""; _ledger.append(1)
assert b"x" * 0 == b""; _ledger.append(1)
assert b"x" * 1 == b"x"; _ledger.append(1)

# Membership — single-byte needle
assert b"x" in b"axc"; _ledger.append(1)
assert b"y" not in b"axc"; _ledger.append(1)
# Multi-byte needle
assert b"bc" in b"abcdef"; _ledger.append(1)
assert b"xy" not in b"abcdef"; _ledger.append(1)

# Equality — same content equal
assert b"abc" == b"abc"; _ledger.append(1)
assert b"" == b""; _ledger.append(1)
assert b"a" == b"a"; _ledger.append(1)
# Inequality — different content unequal
assert b"abc" != b"abcd"; _ledger.append(1)
assert b"abc" != b"abd"; _ledger.append(1)
assert b"" != b"x"; _ledger.append(1)

# len
assert len(b"") == 0; _ledger.append(1)
assert len(b"a") == 1; _ledger.append(1)
assert len(b"hello") == 5; _ledger.append(1)
assert len(b"\x00\x01\x02") == 3; _ledger.append(1)

# b"".join() — empty separator
assert b"".join([b"a", b"b", b"c"]) == b"abc"; _ledger.append(1)
assert b"".join([b"hello", b" ", b"world"]) == b"hello world"; _ledger.append(1)
assert b"".join([]) == b""; _ledger.append(1)

# Non-empty separator join
assert b"-".join([b"a", b"b", b"c"]) == b"a-b-c"; _ledger.append(1)
assert b", ".join([b"x", b"y"]) == b"x, y"; _ledger.append(1)
assert b"|".join([b"only"]) == b"only"; _ledger.append(1)

# bytes(n) — n zero-bytes
assert bytes(0) == b""; _ledger.append(1)
assert bytes(3) == b"\x00\x00\x00"; _ledger.append(1)

# bytes(list_of_int) — each int becomes a byte
assert bytes([97, 98, 99]) == b"abc"; _ledger.append(1)
assert bytes([72, 105]) == b"Hi"; _ledger.append(1)
assert bytes([]) == b""; _ledger.append(1)
assert bytes([0, 1, 2]) == b"\x00\x01\x02"; _ledger.append(1)

# hex / fromhex round-trip
assert b"abc".hex() == "616263"; _ledger.append(1)
assert b"\x00\x01\x02".hex() == "000102"; _ledger.append(1)
assert b"Hello".hex() == "48656c6c6f"; _ledger.append(1)
assert bytes.fromhex("616263") == b"abc"; _ledger.append(1)
assert bytes.fromhex("48656c6c6f") == b"Hello"; _ledger.append(1)
assert bytes.fromhex("") == b""; _ledger.append(1)
# round-trip
b_orig = b"\xde\xad\xbe\xef"
assert bytes.fromhex(b_orig.hex()) == b_orig; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_bytes_operators_slicing_ops {sum(_ledger)} asserts")
