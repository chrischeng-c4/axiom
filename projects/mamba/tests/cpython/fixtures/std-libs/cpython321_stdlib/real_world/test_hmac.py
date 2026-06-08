# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_hmac"
# subject = "cpython321.test_hmac"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_hmac.py"
# status = "filled"
# ///
"""cpython321.test_hmac: execute CPython 3.12 seed test_hmac"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Promoted from the upstream unittest port to an executable AssertionPass seed.
# Surface: hmac (new/digest/hexdigest, compare_digest constant-time, copy).
# All RFC 4231 vectors here use SHA-256, which mamba's hashlib already matches
# in the test_hashlib seed (#3411).
import hmac
import hashlib

_ledger: list[int] = []

# hmac.new + hexdigest produces a 64-char hex string for SHA-256
h = hmac.new(b"key", b"msg", hashlib.sha256)
assert len(h.hexdigest()) == 64, f"sha256 hmac hex length 64, got {len(h.hexdigest())}"
_ledger.append(1)

# hmac.new matches RFC 4231 test case 1 (sha256)
h_rfc1 = hmac.new(b"\x0b" * 20, b"Hi There", hashlib.sha256)
assert h_rfc1.hexdigest() == (
    "b0344c61d8db38535ca8afceaf0bf12b881dc200c9833da726e9376c2e32cff7"
), "RFC 4231 test case 1 sha256 hmac matches reference"
_ledger.append(1)

# update() is equivalent to passing the message in the constructor
h_inc = hmac.new(b"key", digestmod=hashlib.sha256)
h_inc.update(b"msg")
assert h_inc.hexdigest() == h.hexdigest(), (
    "incremental update == one-shot constructor"
)
_ledger.append(1)

# digest() returns 32 raw bytes for SHA-256
raw = h.digest()
assert isinstance(raw, bytes) and len(raw) == 32, (
    f"sha256 hmac digest is 32 bytes, got {type(raw).__name__} len={len(raw)}"
)
_ledger.append(1)

# digest() and hexdigest() agree byte-for-byte
assert raw.hex() == h.hexdigest(), "digest().hex() == hexdigest()"
_ledger.append(1)

# hmac.digest(key, msg, name) one-shot matches the object form
one_shot = hmac.digest(b"key", b"msg", "sha256")
assert one_shot == raw, "hmac.digest one-shot matches hmac.new().digest()"
_ledger.append(1)

# digest_size attribute is 32 for SHA-256
assert h.digest_size == 32, f"sha256 hmac digest_size is 32, got {h.digest_size}"
_ledger.append(1)

# block_size attribute is 64 for SHA-256
assert h.block_size == 64, f"sha256 hmac block_size is 64, got {h.block_size}"
_ledger.append(1)

# name attribute reflects the underlying digest
assert h.name == "hmac-sha256", f"sha256 hmac name is 'hmac-sha256', got {h.name!r}"
_ledger.append(1)

# copy() produces an independent object with the same state
h_copy = h.copy()
assert h_copy.hexdigest() == h.hexdigest(), "copy preserves current digest state"
_ledger.append(1)

# Mutating the copy does not affect the original
h_copy.update(b"more")
assert h_copy.hexdigest() != h.hexdigest(), "copy is independent of the original"
_ledger.append(1)

# compare_digest reports equality for matching byte strings
assert hmac.compare_digest(b"abc", b"abc"), (
    "compare_digest returns truthy for matching inputs"
)
_ledger.append(1)

# compare_digest reports inequality for different byte strings
assert not hmac.compare_digest(b"abc", b"def"), (
    "compare_digest returns falsy for differing inputs"
)
_ledger.append(1)

# compare_digest also handles equal-length distinct-suffix inputs
assert not hmac.compare_digest(b"abcd", b"abce"), (
    "compare_digest detects single trailing-byte differences"
)
_ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_hmac {sum(_ledger)} asserts")
