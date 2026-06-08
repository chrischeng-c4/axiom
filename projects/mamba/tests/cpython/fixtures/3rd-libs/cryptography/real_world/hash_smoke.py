"""Import cryptography and exercise a deterministic SHA-256 hash.

End-user scenario: a downstream package (e.g. PyJWT, requests'
HTTPS path, paramiko, certbot) imports cryptography for a tiny
hashing or version probe. This is the smallest reproducible
"cryptography is loadable and primitives work" gate.

Status: currently registered as `expected_outcome = "xfail"` in
`ecosystem_fixture_manifest.toml` because cryptography ships a
Rust-backed C-extension (`_rust` via PyO3) and mamba's
native-extension loading surface is not ready yet (parent epic
#2526). The fixture exists so the count never silently drops from
the ecosystem report — when mamba can load cryptography's native
core, the same script becomes a required pass without any
fixture-side change.

Random output is not used: the fixture asserts a fixed
SHA-256 known-answer vector and the package's __version__ shape,
so the pass criterion stays deterministic.

DoD (once xfail graduates to pass): exit 0 under both CPython and
mamba. Until then, this script is expected to fail under mamba
and its failure is bucketed as `xfail`, not `required_fail`.
"""

import cryptography
from cryptography.hazmat.primitives import hashes

# -- 1. Version path: cryptography exposes a __version__ string -------------

version = cryptography.__version__
assert isinstance(version, str), (
    f"cryptography.__version__ must be a str, got {type(version).__name__}"
)
assert len(version) > 0, "cryptography.__version__ must be non-empty"
# Version strings always start with a digit (PEP 440-ish), e.g. "42.0.5".
assert version[0].isdigit(), f"cryptography.__version__ must start with a digit, got {version!r}"

# -- 2. SHA-256 known-answer test via the Hash primitive --------------------

# Fixed input → fixed digest. Any bit error in the Rust _hash binding
# would corrupt this output. The expected value is the canonical NIST
# vector for "The quick brown fox jumps over the lazy dog".
kat_input = b"The quick brown fox jumps over the lazy dog"
kat_expected = bytes.fromhex(
    "d7a8fbb307d7809469ca9abcb0082e4f8d5651e46d3cdb762d02d0bf37c9e592"
)

digester = hashes.Hash(hashes.SHA256())
digester.update(kat_input)
digest = digester.finalize()

assert isinstance(digest, bytes), (
    f"Hash.finalize() must return bytes, got {type(digest).__name__}"
)
assert len(digest) == 32, f"SHA-256 digest must be 32 bytes, got {len(digest)}"
assert digest == kat_expected, (
    f"SHA-256 KAT mismatch:\n  got:      {digest.hex()}\n  expected: {kat_expected.hex()}"
)

# -- 3. Chunked-update equivalence ------------------------------------------

# Same payload, fed in two chunks, must yield the same digest. This
# catches state-machine bugs in the Hash wrapper that single-shot
# tests would miss.
chunked = hashes.Hash(hashes.SHA256())
chunked.update(kat_input[:20])
chunked.update(kat_input[20:])
chunked_digest = chunked.finalize()
assert chunked_digest == kat_expected, (
    f"chunked SHA-256 must equal one-shot digest, got {chunked_digest.hex()}"
)

print("ok: cryptography", version, "sha256", digest.hex()[:16], "...")
