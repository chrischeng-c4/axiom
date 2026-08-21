# Operational AssertionPass divergence-spec fixture for the silent
# value-contract divergences of `hasattr(hashlib, 'pbkdf2_hmac')`
# (the documented "hashlib exposes the PBKDF2-HMAC key derivation
# function" — mamba returns False), `type(uuid.uuid4()).__name__`
# (the documented "uuid4() returns a UUID instance" — mamba
# returns 'int' because uuid4 is an int-handle wrapper), `uuid
# .UUID('12345678-1234-5678-1234-567812345678').version` (the
# documented "version returns None for UUIDs whose variant is not
# RFC 4122 — this literal has NCS variant" — mamba returns 5
# ignoring the variant check), `type(hmac.new(b'k', b'm', 'sha256'))
# .__name__` (the documented "hmac.new returns an HMAC instance"
# — mamba returns 'int'), `type(hashlib.md5()).__name__` (the
# documented "hashlib.md5() returns a HASH instance" — mamba
# returns 'int'), `hashlib.blake2b(b'abc').hexdigest()` (the
# documented blake2b(b'abc') digest 'ba80a53f981c4d0d6a2797b69f12
# f6e94c212f14685ac4b74b12bb6fdbffa2d17d87c5392aab792dc252d5de4533
# cc9518d38aa8dbf1925ab92386edd4009923' — mamba returns the sha512
# digest of 'abc' instead, indicating blake2b silently aliases to
# sha512), `type(hashlib.algorithms_guaranteed).__name__` (the
# documented "algorithms_guaranteed is a set" — mamba returns
# 'list'), `uuid.UUID('12345678-1234-5678-1234-567812345678').node`
# (the documented "UUID exposes the 48-bit node field — 95073701
# 484152 for this literal" — mamba returns None), `hashlib.blake2s
# (b'abc').hexdigest()` (the documented blake2s(b'abc') digest
# '508c5e8c327c14e2e1a72ba34eeb452f37458b209ed63a294d999b4c8667
# 5982' — mamba returns the sha256 digest of 'abc' instead,
# indicating blake2s silently aliases to sha256), and
# `uuid.UUID('12345678-1234-5678-1234-567812345678').time_low` (the
# documented "UUID exposes the 32-bit time_low field — 305419896
# (0x12345678) for this literal" — mamba returns None).
# Ten-pack pinned to atomic 266.
#
# Behavioral edges that CONFORM on mamba (secrets — hasattr token_
# bytes/token_hex/token_urlsafe/choice/randbelow/randbits/compare_
# digest/SystemRandom + token_bytes bytes/lengths, token_hex str/
# lengths, token_urlsafe str, choice membership, randbelow/randbits
# range and type, compare_digest True/False. hmac — hasattr new/
# digest/compare_digest/HMAC + hmac.new sha256 known hexdigest,
# digest_size 32, block_size 64, compare_digest True/False.
# hashlib — hasattr md5/sha1/sha256/sha512/sha224/sha384/sha3_256/
# sha3_512/blake2b/blake2s/new/algorithms_available/algorithms_
# guaranteed + md5(b'')/md5(b'abc')/sha1(b'abc')/sha256(b'abc')/
# sha512(b'abc') known hexdigests, sha256.digest_size==32 / .name
# =='sha256', new() matches direct constructor, digest() returns
# bytes, md5/sha256 block_size 64, 'md5' in algorithms_guaranteed,
# 'sha256' in algorithms_available. uuid — hasattr UUID/uuid1/3/4/
# 5/NAMESPACE_DNS/URL/OID + uuid4 str len 36 / two calls distinct
# + UUID literal str/hex roundtrip + bytes len 16) are covered in
# the matching pass fixture
# `test_secrets_hmac_hashlib_uuid_value_ops`.
import hashlib
import hmac
import uuid


_ledger: list[int] = []

# 1) hasattr(hashlib, 'pbkdf2_hmac') — PBKDF2-HMAC KDF
#    (mamba: returns False)
assert hasattr(hashlib, "pbkdf2_hmac") == True; _ledger.append(1)

# 2) type(uuid.uuid4()).__name__ == 'UUID'
#    (mamba: returns 'int' — uuid4 returns int handle)
assert type(uuid.uuid4()).__name__ == "UUID"; _ledger.append(1)

# 3) uuid.UUID('...').version is None for non-RFC-4122 variant
#    (mamba: returns 5 — ignores variant check)
assert uuid.UUID("12345678-1234-5678-1234-567812345678").version is None; _ledger.append(1)

# 4) type(hmac.new(b'k', b'm', 'sha256')).__name__ == 'HMAC'
#    (mamba: returns 'int')
assert type(hmac.new(b"k", b"m", "sha256")).__name__ == "HMAC"; _ledger.append(1)

# 5) type(hashlib.md5()).__name__ == 'HASH'
#    (mamba: returns 'int')
assert type(hashlib.md5()).__name__ == "HASH"; _ledger.append(1)

# 6) hashlib.blake2b('abc') has its own digest, not sha512's
#    (mamba: silently aliases to sha512, returning sha512's digest)
assert hashlib.blake2b(b"abc").hexdigest() == "ba80a53f981c4d0d6a2797b69f12f6e94c212f14685ac4b74b12bb6fdbffa2d17d87c5392aab792dc252d5de4533cc9518d38aa8dbf1925ab92386edd4009923"; _ledger.append(1)

# 7) type(hashlib.algorithms_guaranteed).__name__ == 'set'
#    (mamba: returns 'list')
assert type(hashlib.algorithms_guaranteed).__name__ == "set"; _ledger.append(1)

# 8) uuid.UUID('...').node == 95073701484152
#    (mamba: returns None — node field not parsed)
assert uuid.UUID("12345678-1234-5678-1234-567812345678").node == 95073701484152; _ledger.append(1)

# 9) hashlib.blake2s('abc') has its own digest, not sha256's
#    (mamba: silently aliases blake2s to sha256, returning sha256's digest)
assert hashlib.blake2s(b"abc").hexdigest() == "508c5e8c327c14e2e1a72ba34eeb452f37458b209ed63a294d999b4c86675982"; _ledger.append(1)

# 10) uuid.UUID('...').time_low == 305419896  (== 0x12345678)
#     (mamba: returns None — time_low field not parsed)
assert uuid.UUID("12345678-1234-5678-1234-567812345678").time_low == 305419896; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_secrets_hmac_hashlib_uuid_silent {sum(_ledger)} asserts")
