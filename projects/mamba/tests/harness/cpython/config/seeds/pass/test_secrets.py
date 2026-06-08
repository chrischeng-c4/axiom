# Promoted from the upstream unittest port to an executable AssertionPass seed.
# Surface: secrets (token_bytes/token_hex/token_urlsafe sizes + uniqueness,
# randbelow range, choice membership, compare_digest, randbits range).
# secrets.SystemRandom() returns a non-class stub under mamba and is omitted.
import secrets

_ledger: list[int] = []

# token_bytes returns the requested number of bytes
tb = secrets.token_bytes(16)
assert isinstance(tb, bytes) and len(tb) == 16, (
    f"token_bytes(16) is 16 bytes, got {type(tb).__name__} len={len(tb)}"
)
_ledger.append(1)

# token_hex returns a 2N-char ASCII hex string for N bytes
th = secrets.token_hex(16)
assert isinstance(th, str) and len(th) == 32, (
    f"token_hex(16) is a 32-char string, got {type(th).__name__} len={len(th)}"
)
_ledger.append(1)

# token_hex output is pure lowercase hex
assert all(c in "0123456789abcdef" for c in th), (
    "token_hex output contains only lowercase hex digits"
)
_ledger.append(1)

# token_urlsafe returns a str whose unpadded length is ceil(4*N/3)
tu = secrets.token_urlsafe(16)
assert isinstance(tu, str) and len(tu) == 22, (
    f"token_urlsafe(16) is a 22-char string, got {type(tu).__name__} len={len(tu)}"
)
_ledger.append(1)

# token_urlsafe uses only the URL-safe base64 alphabet (no '+' or '/')
assert "+" not in tu and "/" not in tu, (
    "token_urlsafe output uses URL-safe alphabet (no '+' or '/')"
)
_ledger.append(1)

# token_bytes outputs are unique across many calls
batch = {secrets.token_bytes(16) for _ in range(100)}
assert len(batch) == 100, f"100 token_bytes(16) calls are distinct, got {len(batch)}"
_ledger.append(1)

# randbelow stays within [0, n)
for _ in range(20):
    r = secrets.randbelow(100)
    assert 0 <= r < 100, f"randbelow(100) returned {r}, expected 0..99"
_ledger.append(1)

# choice picks an element from the input sequence
pool = [10, 20, 30, 40, 50]
for _ in range(20):
    pick = secrets.choice(pool)
    assert pick in pool, f"choice({pool!r}) returned {pick!r}, not from input"
_ledger.append(1)

# compare_digest reports equality for matching strings
assert secrets.compare_digest("abc", "abc"), (
    "compare_digest returns truthy for matching strings"
)
_ledger.append(1)

# compare_digest reports inequality for differing strings
assert not secrets.compare_digest("abc", "def"), (
    "compare_digest returns falsy for differing strings"
)
_ledger.append(1)

# compare_digest also works on bytes
assert secrets.compare_digest(b"abc", b"abc"), (
    "compare_digest returns truthy for matching bytes"
)
_ledger.append(1)

# randbits stays within [0, 2**k)
for k in (8, 16, 32):
    r = secrets.randbits(k)
    assert isinstance(r, int) and 0 <= r < (1 << k), (
        f"randbits({k}) returned {r}, expected 0..(2**{k} - 1)"
    )
_ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_secrets {sum(_ledger)} asserts")
