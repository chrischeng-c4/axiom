"""Hot-loop bench for `secrets.token_bytes` / `token_hex` / `token_urlsafe`.

End-user scenario: a session-token issuer churns through token generation
at the edge of a web request hot path. Each request mints a 32-byte raw
token, a hex form (for storage), and a URL-safe form (for cookies). The
inner work is OS RNG draw + encoding — short and per-call dominated by
dispatch overhead until the iteration count amortizes startup noise.

Tier: `native-shim io-light` (target mamba/cpython >= 1.0x — secrets
in CPython is a thin Python wrapper around `os.urandom` + `binascii`
encoders, so the absolute ceiling is bounded by the syscall + encode
cost; mamba's edge is removing the per-call Python-attribute lookup
overhead around three short hot calls).

Hoist convention (per #2097 + CLAUDE.md note): all three module attrs
hoisted to locals before the loop, so the published ratio reflects the
underlying RNG + encoding cost rather than module-attr lookup speed.

DoD: exits 0 under both CPython and mamba; the cross-runtime bench
and reports the ratio. Floor is 1.0x per #1265 Goal 2.
"""

import secrets

# Hoist module attributes outside the loop — see CLAUDE.md note + #2097.
# Without hoisting, mamba's module-attr lookup at the call site is ~5x
# slower than the hoisted form, biasing the published ratio.
token_bytes = secrets.token_bytes
token_hex = secrets.token_hex
token_urlsafe = secrets.token_urlsafe

# Tens of thousands of iterations to swamp Python/mamba startup overhead
# while staying within tens-of-ms steady-state wall time on both runtimes.
ITERS = 10_000
NBYTES = 32  # canonical session-token width (matches DEFAULT_ENTROPY).

total_len = 0
# Internal-time marker — see hashlib/digest_1mb.py rationale. Wall-time
# ratio is biased by Python startup overhead (~200ms CPython vs ~5ms
# mamba); this marker captures the steady-state per-call cost.
for _ in range(ITERS):
    raw = token_bytes(NBYTES)
    h = token_hex(NBYTES)
    u = token_urlsafe(NBYTES)
    total_len += len(raw) + len(h) + len(u)

# Invariant lengths: raw=32, hex=64, urlsafe=43 (per RFC 4648 base64url
# of 32 bytes, no padding). Sum = 139 per iter. Use subtraction-equals-
# zero (per boxed-accumulator-int-equality memory entry).
expected = ITERS * (NBYTES + 2 * NBYTES + 43)
diff = total_len - expected
assert diff == 0, f"token length mismatch: total={total_len} expected={expected} diff={diff}"
print("token_bytes_hot:", total_len)
