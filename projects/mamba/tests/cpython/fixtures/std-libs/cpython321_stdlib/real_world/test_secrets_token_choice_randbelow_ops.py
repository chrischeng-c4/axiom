# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_secrets_token_choice_randbelow_ops"
# subject = "cpython321.test_secrets_token_choice_randbelow_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_secrets_token_choice_randbelow_ops.py"
# status = "filled"
# ///
"""cpython321.test_secrets_token_choice_randbelow_ops: execute CPython 3.12 seed test_secrets_token_choice_randbelow_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for the `secrets` stdlib module
# — the cryptographically-secure random surface used for session
# tokens, API keys, CSRF tokens, password-reset URLs, and any other
# adversary-resistant randomness path. Surface pins the matching
# subset between mamba and CPython on token_bytes / token_hex /
# token_urlsafe (output type + length contract), compare_digest
# (constant-time string/bytes equality), choice (uniform sampling
# from a list), randbelow (bounded integer sampling), randbits
# (bit-width-bounded integer sampling), and module attribute
# discipline. Complementary to the `random` module coverage in
# `test_random_*_ops.py`; this seed is specifically the
# crypto-grade entropy path required by RFC 4122 / RFC 7517 /
# OAuth token issuance.
#
# Surface:
#   • secrets.token_bytes(n: int) → bytes of length n
#       — n=1, 4, 8, 16, 32, 64 all preserve the length contract;
#   • secrets.token_hex(n: int) → str of length 2*n (hex chars);
#       — n=1, 4, 8, 16, 32 all preserve the 2× length contract;
#   • secrets.token_urlsafe(n: int) → str
#       — base64url-encoded n-byte token;
#   • secrets.compare_digest(a, b) → bool
#       — constant-time string/bytes equality;
#       — symmetric across str and bytes inputs;
#       — empty inputs compare equal;
#   • secrets.choice(seq) → element
#       — uniform sampling from a list;
#       — output type matches element type;
#   • secrets.randbelow(n: int) → int in [0, n)
#       — n=1, 10, 100, 1000, 2**32 all bounded;
#   • secrets.randbits(k: int) → int in [0, 2**k)
#       — k=1, 8, 16, 32 preserve the bit-width contract
#         (k>=64 hits mamba's int-overflow regime — excluded here);
#   • module-level attribute discipline — every helper hasattr +
#     callable, module name == 'secrets'.
import secrets
_ledger: list[int] = []

# token_bytes — type and length contract across various n
for _n in [1, 4, 8, 16, 32, 64]:
    _b = secrets.token_bytes(_n)
    assert isinstance(_b, bytes); _ledger.append(1)
    assert len(_b) == _n; _ledger.append(1)

# token_hex — type and length contract (output is 2*n hex chars)
for _n in [1, 4, 8, 16, 32]:
    _h = secrets.token_hex(_n)
    assert isinstance(_h, str); _ledger.append(1)
    assert len(_h) == 2 * _n; _ledger.append(1)

# token_hex — output is pure hex (lowercase 0-9 a-f)
_hex_chars = set("0123456789abcdef")
_h = secrets.token_hex(16)
for _c in _h:
    assert _c in _hex_chars; _ledger.append(1)

# token_urlsafe — output type is str
for _n in [1, 4, 8, 16, 32]:
    _u = secrets.token_urlsafe(_n)
    assert isinstance(_u, str); _ledger.append(1)

# token_urlsafe — output stays within the base64url alphabet
# (A-Z a-z 0-9 - _ — no padding for token_urlsafe)
_b64url_chars = set("ABCDEFGHIJKLMNOPQRSTUVWXYZ"
                    "abcdefghijklmnopqrstuvwxyz"
                    "0123456789-_")
_u = secrets.token_urlsafe(32)
for _c in _u:
    assert _c in _b64url_chars; _ledger.append(1)

# compare_digest — equal strings
assert secrets.compare_digest("abc", "abc") == True; _ledger.append(1)
assert secrets.compare_digest("hello world", "hello world") == True; _ledger.append(1)
assert secrets.compare_digest("", "") == True; _ledger.append(1)

# compare_digest — unequal strings
assert secrets.compare_digest("abc", "xyz") == False; _ledger.append(1)
assert secrets.compare_digest("hello", "world") == False; _ledger.append(1)
assert secrets.compare_digest("a", "ab") == False; _ledger.append(1)
assert secrets.compare_digest("abc", "abd") == False; _ledger.append(1)

# compare_digest — equal bytes
assert secrets.compare_digest(b"abc", b"abc") == True; _ledger.append(1)
assert secrets.compare_digest(b"hello world", b"hello world") == True; _ledger.append(1)
assert secrets.compare_digest(b"", b"") == True; _ledger.append(1)

# compare_digest — unequal bytes
assert secrets.compare_digest(b"abc", b"xyz") == False; _ledger.append(1)
assert secrets.compare_digest(b"\x00\x01", b"\x00\x02") == False; _ledger.append(1)
assert secrets.compare_digest(b"a", b"ab") == False; _ledger.append(1)

# choice — uniform sampling from list of ints
_options_int = [1, 2, 3, 4, 5]
for _ in range(10):
    _r = secrets.choice(_options_int)
    assert _r in _options_int; _ledger.append(1)

# choice — uniform sampling from list of strings, output type str
_options_str = ["alpha", "beta", "gamma", "delta"]
for _ in range(10):
    _r = secrets.choice(_options_str)
    assert _r in _options_str; _ledger.append(1)
    assert isinstance(_r, str); _ledger.append(1)

# choice — single-element list deterministic
assert secrets.choice([42]) == 42; _ledger.append(1)
assert secrets.choice(["only"]) == "only"; _ledger.append(1)

# randbelow — output is in [0, n)
for _n in [1, 10, 100, 1000, 1 << 32]:
    _r = secrets.randbelow(_n)
    assert isinstance(_r, int); _ledger.append(1)
    assert 0 <= _r < _n; _ledger.append(1)

# randbelow(1) is always 0
for _ in range(5):
    assert secrets.randbelow(1) == 0; _ledger.append(1)

# randbits — output is in [0, 2**k) for small k
# (k>=64 hits mamba's int-overflow regime — excluded)
for _k in [1, 8, 16, 32]:
    _r = secrets.randbits(_k)
    assert isinstance(_r, int); _ledger.append(1)
    assert 0 <= _r < (1 << _k); _ledger.append(1)

# randbits(1) is 0 or 1
for _ in range(5):
    _r = secrets.randbits(1)
    assert _r in (0, 1); _ledger.append(1)

# randbits(8) is in [0, 256)
for _ in range(5):
    _r = secrets.randbits(8)
    assert 0 <= _r < 256; _ledger.append(1)

# Module-level helper-function attribute discipline
for _name in ['token_bytes', 'token_hex', 'token_urlsafe',
              'compare_digest', 'choice', 'randbelow', 'randbits']:
    assert hasattr(secrets, _name); _ledger.append(1)
    assert callable(getattr(secrets, _name)); _ledger.append(1)

# Module name discipline
assert secrets.__name__ == 'secrets'; _ledger.append(1)

# token_bytes — different calls produce different output (very high probability)
_a = secrets.token_bytes(32)
_b = secrets.token_bytes(32)
assert _a != _b; _ledger.append(1)

# token_hex — different calls produce different output
_a = secrets.token_hex(32)
_b = secrets.token_hex(32)
assert _a != _b; _ledger.append(1)

# token_bytes(0) — zero-length contract
assert secrets.token_bytes(0) == b''; _ledger.append(1)
assert len(secrets.token_bytes(0)) == 0; _ledger.append(1)

# token_hex(0) — zero-length contract
assert secrets.token_hex(0) == ''; _ledger.append(1)
assert len(secrets.token_hex(0)) == 0; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_secrets_token_choice_randbelow_ops {sum(_ledger)} asserts")
