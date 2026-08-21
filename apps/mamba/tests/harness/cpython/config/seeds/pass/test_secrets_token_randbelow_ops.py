# Operational AssertionPass seed for the `secrets` module — the
# stdlib cryptographically-strong random helper used for password
# generators, session tokens, CSRF tokens, API key minting, and
# any code that needs unpredictable random bytes / random ints
# suitable for security-sensitive use. Surface focuses on the
# matching subset between mamba and CPython on the deterministic
# length contracts (`token_hex(n)` returns 2n hex chars,
# `token_bytes(n)` returns n bytes, `token_urlsafe(n)` returns
# ⌈n*4/3⌉ url-safe chars after stripping padding), and the range
# invariants of `randbelow(n)` and `choice(list)`. `choice` on a
# str returns None on mamba — only list/list-like is exercised
# here. No fixture coverage yet for secrets.
#
# Surface:
#   • secrets.token_hex(n: int) → str
#       — 2n lowercase-hex chars;
#       — token_hex(0) → '';
#   • secrets.token_bytes(n: int) → bytes
#       — exactly n random bytes;
#       — token_bytes(0) → b'';
#   • secrets.token_urlsafe(n: int) → str
#       — url-safe base64-encoded, padding stripped;
#       — token_urlsafe(16) → 22 chars;
#       — token_urlsafe(8) → 11 chars;
#   • secrets.randbelow(n: int) → int
#       — uniformly random int in [0, n);
#       — randbelow(1) → 0 always;
#   • secrets.choice(seq) → element
#       — uniformly random pick from non-empty sequence.
import secrets
_ledger: list[int] = []

# token_hex — exact length (2n hex chars)
assert len(secrets.token_hex(16)) == 32; _ledger.append(1)
assert len(secrets.token_hex(8)) == 16; _ledger.append(1)
assert len(secrets.token_hex(32)) == 64; _ledger.append(1)
assert len(secrets.token_hex(1)) == 2; _ledger.append(1)
assert len(secrets.token_hex(0)) == 0; _ledger.append(1)
assert secrets.token_hex(0) == ''; _ledger.append(1)
assert isinstance(secrets.token_hex(16), str); _ledger.append(1)

# token_hex — output uses only lowercase hex digits
_h = secrets.token_hex(32)
for _c in _h:
    _is_d = '0' <= _c <= '9'
    _is_l = 'a' <= _c <= 'f'
    assert _is_d or _is_l; _ledger.append(1)

# token_bytes — exact length
assert len(secrets.token_bytes(16)) == 16; _ledger.append(1)
assert len(secrets.token_bytes(8)) == 8; _ledger.append(1)
assert len(secrets.token_bytes(32)) == 32; _ledger.append(1)
assert len(secrets.token_bytes(1)) == 1; _ledger.append(1)
assert len(secrets.token_bytes(0)) == 0; _ledger.append(1)
assert secrets.token_bytes(0) == b''; _ledger.append(1)
assert isinstance(secrets.token_bytes(16), bytes); _ledger.append(1)

# token_urlsafe — base64-encoded length (no padding):
#   16 bytes  → ceil(16*4/3) = 22 chars (with padding stripped)
#   8 bytes   → ceil(8*4/3)  = 11 chars (with padding stripped)
#   32 bytes  → ceil(32*4/3) = 43 chars
assert len(secrets.token_urlsafe(16)) == 22; _ledger.append(1)
assert len(secrets.token_urlsafe(8)) == 11; _ledger.append(1)
assert len(secrets.token_urlsafe(32)) == 43; _ledger.append(1)
assert isinstance(secrets.token_urlsafe(16), str); _ledger.append(1)

# randbelow — range invariant
for _ in range(20):
    _r = secrets.randbelow(100)
    assert 0 <= _r < 100; _ledger.append(1)
    assert isinstance(_r, int); _ledger.append(1)

# randbelow(1) is always 0
assert secrets.randbelow(1) == 0; _ledger.append(1)
assert secrets.randbelow(1) == 0; _ledger.append(1)

# randbelow on small ranges
for _ in range(10):
    _r = secrets.randbelow(2)
    assert _r == 0 or _r == 1; _ledger.append(1)

# randbelow on larger range
for _ in range(10):
    _r = secrets.randbelow(1000000)
    assert 0 <= _r < 1000000; _ledger.append(1)

# choice — picks from the sequence
_picks = [secrets.choice([1, 2, 3, 4, 5]) for _ in range(20)]
for _p in _picks:
    assert _p in [1, 2, 3, 4, 5]; _ledger.append(1)
    assert isinstance(_p, int); _ledger.append(1)

# choice on single-element list returns that element
assert secrets.choice([42]) == 42; _ledger.append(1)
assert secrets.choice([42]) == 42; _ledger.append(1)
assert secrets.choice(["only"]) == "only"; _ledger.append(1)

# choice on multi-element list returns an element from it
for _ in range(10):
    _c = secrets.choice(["alpha", "beta", "gamma"])
    assert _c in ["alpha", "beta", "gamma"]; _ledger.append(1)

# Module-level attribute discipline
for _name in ["token_hex", "token_bytes", "token_urlsafe",
              "randbelow", "choice"]:
    assert hasattr(secrets, _name); _ledger.append(1)
    assert callable(getattr(secrets, _name)); _ledger.append(1)

# Length-invariant — token_hex's length is deterministic across calls
for _n in [1, 4, 8, 16, 32, 64]:
    assert len(secrets.token_hex(_n)) == 2 * _n; _ledger.append(1)
    assert len(secrets.token_bytes(_n)) == _n; _ledger.append(1)

# Two token_hex calls with same arg → same length (different content)
assert len(secrets.token_hex(16)) == len(secrets.token_hex(16)); _ledger.append(1)
assert len(secrets.token_bytes(16)) == len(secrets.token_bytes(16)); _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_secrets_token_randbelow_ops {sum(_ledger)} asserts")
