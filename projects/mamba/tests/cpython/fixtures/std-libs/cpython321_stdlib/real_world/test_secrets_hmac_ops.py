# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_secrets_hmac_ops"
# subject = "cpython321.test_secrets_hmac_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_secrets_hmac_ops.py"
# status = "filled"
# ///
"""cpython321.test_secrets_hmac_ops: execute CPython 3.12 seed test_secrets_hmac_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for `secrets` and `hmac` modules.
# Surface: secrets.token_hex/token_bytes length invariants,
# hmac.new(..).hexdigest() canonical-vector for sha256("hello").
# Companion to stub/test_secrets.py + stub/test_hmac.py — vendored
# unittest seeds.
import secrets
import hmac
import hashlib
_ledger: list[int] = []
# token_hex(n) → 2n hex chars
assert len(secrets.token_hex(8)) == 16; _ledger.append(1)
assert len(secrets.token_hex(16)) == 32; _ledger.append(1)
assert len(secrets.token_bytes(16)) == 16; _ledger.append(1)
assert len(secrets.token_bytes(32)) == 32; _ledger.append(1)
# hex chars only
for ch in secrets.token_hex(4):
    assert ch in "0123456789abcdef"
_ledger.append(1)
# hmac
h = hmac.new(b"secret", b"message", hashlib.sha256).hexdigest()
assert len(h) == 64; _ledger.append(1)
assert h == "8b5f48702995c1598c573db1e21866a9b825d4a794d169d7060a03605796360b"; _ledger.append(1)
# different keys differ
h2 = hmac.new(b"other", b"message", hashlib.sha256).hexdigest()
assert h2 != h; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: test_secrets_hmac_ops {sum(_ledger)} asserts")
