# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_secrets_token_choice_ops"
# subject = "cpython321.test_secrets_token_choice_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_secrets_token_choice_ops.py"
# status = "filled"
# ///
"""cpython321.test_secrets_token_choice_ops: execute CPython 3.12 seed test_secrets_token_choice_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for secrets.token_*/choice/randbits/
# randbelow/compare_digest surface. Surface: secrets.choice on
# list[int]/list[str] returns a member; secrets.token_bytes(n) returns
# exactly n bytes (and the default-no-arg form returns a positive-length
# bytes); secrets.token_hex(n) returns 2*n hex chars (and the default
# form returns a positive-length str); secrets.token_urlsafe(n) returns
# a str of length >= n; secrets.randbits(k) returns a value in [0, 2**k);
# secrets.randbelow(n) returns a value in [0, n); secrets.randbelow(1)
# is always 0; secrets.compare_digest returns True for equal str/bytes
# and False for differing; two successive token_bytes/token_hex calls
# return different values (entropy contract). Companion to
# test_secrets_ops and test_secrets_hmac_ops.
import secrets
_ledger: list[int] = []

# choice — returns a member of the source sequence
assert secrets.choice([1, 2, 3]) in [1, 2, 3]; _ledger.append(1)
assert secrets.choice(["a", "b", "c"]) in ["a", "b", "c"]; _ledger.append(1)

# token_bytes — explicit-n length matches, default form is positive-length bytes
assert len(secrets.token_bytes(16)) == 16; _ledger.append(1)
assert len(secrets.token_bytes(32)) == 32; _ledger.append(1)
assert isinstance(secrets.token_bytes(8), bytes); _ledger.append(1)
assert isinstance(secrets.token_bytes(), bytes); _ledger.append(1)
assert len(secrets.token_bytes()) > 0; _ledger.append(1)

# token_hex — explicit-n produces 2*n hex chars, default form is positive-length str
assert len(secrets.token_hex(16)) == 32; _ledger.append(1)
assert len(secrets.token_hex(8)) == 16; _ledger.append(1)
assert isinstance(secrets.token_hex(8), str); _ledger.append(1)
assert isinstance(secrets.token_hex(), str); _ledger.append(1)
assert len(secrets.token_hex()) > 0; _ledger.append(1)

# token_urlsafe — length is >= n (urlsafe encoding may expand)
assert len(secrets.token_urlsafe(16)) >= 16; _ledger.append(1)
assert isinstance(secrets.token_urlsafe(16), str); _ledger.append(1)

# randbits / randbelow — range bounds
assert 0 <= secrets.randbits(8) < 256; _ledger.append(1)
assert 0 <= secrets.randbits(16) < 65536; _ledger.append(1)
assert 0 <= secrets.randbelow(100) < 100; _ledger.append(1)
assert 0 <= secrets.randbelow(10) < 10; _ledger.append(1)
assert secrets.randbelow(1) == 0; _ledger.append(1)

# compare_digest — equal and unequal for str and bytes
assert secrets.compare_digest("abc", "abc") == True; _ledger.append(1)
assert secrets.compare_digest("abc", "abd") == False; _ledger.append(1)
assert secrets.compare_digest(b"abc", b"abc") == True; _ledger.append(1)
assert secrets.compare_digest(b"abc", b"abd") == False; _ledger.append(1)

# Successive draws differ with overwhelming probability
assert secrets.token_bytes(16) != secrets.token_bytes(16); _ledger.append(1)
assert secrets.token_hex(16) != secrets.token_hex(16); _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_secrets_token_choice_ops {sum(_ledger)} asserts")
