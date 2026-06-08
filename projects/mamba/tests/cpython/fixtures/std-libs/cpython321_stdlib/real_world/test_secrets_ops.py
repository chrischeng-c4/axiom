# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_secrets_ops"
# subject = "cpython321.test_secrets_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_secrets_ops.py"
# status = "filled"
# ///
"""cpython321.test_secrets_ops: execute CPython 3.12 seed test_secrets_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for the secrets stdlib module.
# Surface: token_hex(n) returns a string of length 2*n (n bytes → 2n
# hex chars), default length 32 bytes → 64 hex chars; token_bytes(n)
# returns a `bytes` of exactly n bytes; token_urlsafe(n) returns a
# `str`; choice(seq) picks a member of the sequence; randbelow(n)
# returns an int in [0, n); compare_digest performs a constant-time
# equality check (equal pairs return True, unequal return False, and
# bytes pairs are also accepted); randbits(k) returns an int that
# fits in k bits; consecutive token calls produce distinct values.
import secrets
_ledger: list[int] = []

# token_hex(n) → 2n hex chars
t8 = secrets.token_hex(8)
assert len(t8) == 16; _ledger.append(1)
# every char of the hex string is a hex digit
hex_chars = "0123456789abcdef"
assert all(c in hex_chars for c in t8); _ledger.append(1)

# token_hex(16) → 32 hex chars
t16 = secrets.token_hex(16)
assert len(t16) == 32; _ledger.append(1)
assert all(c in hex_chars for c in t16); _ledger.append(1)

# token_hex with no argument → default 32 bytes = 64 hex chars
td = secrets.token_hex()
assert len(td) == 64; _ledger.append(1)
assert all(c in hex_chars for c in td); _ledger.append(1)

# token_bytes(n) → bytes of length n
b16 = secrets.token_bytes(16)
assert len(b16) == 16; _ledger.append(1)
assert type(b16).__name__ == "bytes"; _ledger.append(1)
# Larger length
b64 = secrets.token_bytes(64)
assert len(b64) == 64; _ledger.append(1)

# token_urlsafe(n) → str with positive length
u = secrets.token_urlsafe(16)
assert type(u).__name__ == "str"; _ledger.append(1)
assert len(u) > 0; _ledger.append(1)

# choice on a list returns a member of the list
c = secrets.choice([1, 2, 3, 4, 5])
assert c in [1, 2, 3, 4, 5]; _ledger.append(1)
# choice on a single-element list always returns that element
c2 = secrets.choice([42])
assert c2 == 42; _ledger.append(1)

# randbelow(n) → 0 ≤ r < n
for _i in range(10):
    r = secrets.randbelow(10)
    assert 0 <= r < 10; _ledger.append(1)
# randbelow(1) is always 0
assert secrets.randbelow(1) == 0; _ledger.append(1)

# compare_digest — equal strings return True
assert secrets.compare_digest("abc", "abc") == True; _ledger.append(1)
# unequal strings return False
assert secrets.compare_digest("abc", "abd") == False; _ledger.append(1)
assert secrets.compare_digest("a", "b") == False; _ledger.append(1)
# bytes pairs are accepted too
assert secrets.compare_digest(b"abc", b"abc") == True; _ledger.append(1)
assert secrets.compare_digest(b"abc", b"abd") == False; _ledger.append(1)
# empty strings are equal
assert secrets.compare_digest("", "") == True; _ledger.append(1)

# Consecutive token calls produce distinct values (the whole point of
# a CSPRNG — collision risk is astronomically low for 16-byte tokens)
t_a = secrets.token_hex(16)
t_b = secrets.token_hex(16)
assert t_a != t_b; _ledger.append(1)
# Same for token_bytes
ba = secrets.token_bytes(16)
bb = secrets.token_bytes(16)
assert ba != bb; _ledger.append(1)

# randbits(k) returns an int that fits in k bits — 0 ≤ r < 2**k
for _j in range(5):
    rb = secrets.randbits(8)
    assert 0 <= rb < 256; _ledger.append(1)
    rb16 = secrets.randbits(16)
    assert 0 <= rb16 < 65536; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: test_secrets_ops {sum(_ledger)} asserts")
