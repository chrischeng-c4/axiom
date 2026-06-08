# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_keyword_hashlib_hmac_secrets_value_ops"
# subject = "cpython321.test_keyword_hashlib_hmac_secrets_value_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_keyword_hashlib_hmac_secrets_value_ops.py"
# status = "filled"
# ///
"""cpython321.test_keyword_hashlib_hmac_secrets_value_ops: execute CPython 3.12 seed test_keyword_hashlib_hmac_secrets_value_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for the value contract of four
# bootstrap stdlib modules used by every parser / crypto-digest /
# session-token pipeline: `keyword` (the iskeyword / kwlist /
# softkwlist parser-syntax predicates), `hashlib` (the documented
# md5 / sha1 / sha256 / sha512 hexdigest byte-exact contract and
# digest_size / block_size / name introspection), `hmac` (the
# new / hexdigest / digest_size / block_size / name / compare_digest
# message-authentication surface), and `secrets` (the token_bytes
# / token_hex / token_urlsafe / randbelow / choice cryptographic-
# random surface).
#
# The matching subset between mamba and CPython is the byte-exact
# digest layer + parser predicate + arity contract: keyword.
# iskeyword returns True for "for" / "class" / "True", False for
# arbitrary identifiers; keyword.kwlist is a non-empty list with
# "for" present; keyword.softkwlist contains "match" (PEP 634
# soft keyword); hashlib.md5(b"hello").hexdigest() ==
# "5d41402abc4b2a76b9719d911017c592" + digest_size == 16 +
# block_size == 64 + name == "md5"; same byte-exact contracts for
# sha1 / sha256 / sha512; "md5" and "sha256" are in
# hashlib.algorithms_guaranteed; hmac.new(b"key", b"msg",
# "sha256").hexdigest() == "2d93cbc1be167bcb1637a4a23cbff01a7878
# f0c50ee833954ea5221bb1b8c628" + digest_size == 32 + block_size
# == 64 + name == "hmac-sha256"; hmac.compare_digest("abc","abc")
# is True; secrets.token_bytes(n) returns `bytes` of length n;
# secrets.token_hex(n) returns `str` of length 2*n;
# secrets.token_urlsafe(n) returns non-empty `str`; secrets.
# randbelow(100) returns int < 100; secrets.choice([1,2,3])
# returns an `int` from the sequence.
#
# Surface in this fixture:
#   • keyword.iskeyword("for") is True;
#   • keyword.iskeyword("class") is True;
#   • keyword.iskeyword("True") is True;
#   • keyword.iskeyword("foo") is False;
#   • type(keyword.kwlist) is list;
#   • len(keyword.kwlist) > 0;
#   • "for" in keyword.kwlist;
#   • "foo" not in keyword.kwlist;
#   • hasattr(keyword, "softkwlist") and "match" in softkwlist;
#   • hashlib.md5(b"hello").hexdigest() ==
#     "5d41402abc4b2a76b9719d911017c592";
#   • hashlib.md5(b"hello").digest_size == 16;
#   • hashlib.md5(b"hello").block_size == 64;
#   • hashlib.md5(b"hello").name == "md5";
#   • hashlib.sha1(b"hello").hexdigest() ==
#     "aaf4c61ddcc5e8a2dabede0f3b482cd9aea9434d";
#   • hashlib.sha1.digest_size == 20, block_size == 64;
#   • hashlib.sha256(b"hello").hexdigest() ==
#     "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824";
#   • hashlib.sha256.digest_size == 32, block_size == 64;
#   • hashlib.sha512.digest_size == 64, block_size == 128;
#   • "md5" / "sha256" in hashlib.algorithms_guaranteed;
#   • hmac.new(b"key", b"msg", "sha256").hexdigest() == "2d93cbc1
#     be167bcb1637a4a23cbff01a7878f0c50ee833954ea5221bb1b8c628";
#   • hmac.new.digest_size == 32, block_size == 64,
#     name == "hmac-sha256";
#   • hmac.compare_digest("abc", "abc") is True;
#   • hmac.compare_digest("abc", "abd") is False;
#   • secrets.token_bytes(8) is `bytes` of length 8;
#   • secrets.token_hex(8) is `str` of length 16;
#   • secrets.token_urlsafe(8) is non-empty `str`;
#   • type(secrets.randbelow(100)) is int and < 100;
#   • type(secrets.choice([1,2,3])) is int.
#
# Behavioral edges that DIVERGE on mamba (hashlib.
# algorithms_guaranteed container type — `set` on CPython, `list`
# on mamba; hmac.HMAC class identity; random.Random /
# SystemRandom class identity + seed-deterministic output;
# enum.Enum / IntEnum / Flag / IntFlag class identity +
# Enum-subclass `.name` / `.value` member surface + enum.auto
# helper) are covered in
# `lang_random_enum_hmac_class_silent`.
import keyword
import hashlib
import hmac
import secrets

_ledger: list[int] = []

# 1) keyword.iskeyword — parser predicate
assert keyword.iskeyword("for") == True; _ledger.append(1)
assert keyword.iskeyword("class") == True; _ledger.append(1)
assert keyword.iskeyword("True") == True; _ledger.append(1)
assert keyword.iskeyword("foo") == False; _ledger.append(1)

# 2) keyword.kwlist — reserved-word list
assert type(keyword.kwlist).__name__ == "list"; _ledger.append(1)
assert len(keyword.kwlist) > 0; _ledger.append(1)
assert "for" in keyword.kwlist; _ledger.append(1)
assert "foo" not in keyword.kwlist; _ledger.append(1)

# 3) keyword.softkwlist — PEP 634 soft keywords
assert hasattr(keyword, "softkwlist"); _ledger.append(1)
assert "match" in keyword.softkwlist; _ledger.append(1)

# 4) hashlib.md5 — RFC 1321 hex digest
_md5 = hashlib.md5(b"hello")
assert _md5.hexdigest() == "5d41402abc4b2a76b9719d911017c592"; _ledger.append(1)
assert _md5.digest_size == 16; _ledger.append(1)
assert _md5.block_size == 64; _ledger.append(1)
assert _md5.name == "md5"; _ledger.append(1)

# 5) hashlib.sha1 — RFC 3174 hex digest
_sha1 = hashlib.sha1(b"hello")
assert _sha1.hexdigest() == "aaf4c61ddcc5e8a2dabede0f3b482cd9aea9434d"; _ledger.append(1)
assert _sha1.digest_size == 20; _ledger.append(1)
assert _sha1.block_size == 64; _ledger.append(1)

# 6) hashlib.sha256 — FIPS 180-4 hex digest
_sha256 = hashlib.sha256(b"hello")
assert _sha256.hexdigest() == "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"; _ledger.append(1)
assert _sha256.digest_size == 32; _ledger.append(1)
assert _sha256.block_size == 64; _ledger.append(1)

# 7) hashlib.sha512 — FIPS 180-4 sizes
_sha512 = hashlib.sha512(b"hello")
assert _sha512.digest_size == 64; _ledger.append(1)
assert _sha512.block_size == 128; _ledger.append(1)

# 8) hashlib.algorithms_guaranteed — membership predicate
assert "md5" in hashlib.algorithms_guaranteed; _ledger.append(1)
assert "sha1" in hashlib.algorithms_guaranteed; _ledger.append(1)
assert "sha256" in hashlib.algorithms_guaranteed; _ledger.append(1)
assert "sha512" in hashlib.algorithms_guaranteed; _ledger.append(1)

# 9) hmac.new — RFC 2104 HMAC-SHA256
_hm = hmac.new(b"key", b"msg", "sha256")
assert _hm.hexdigest() == "2d93cbc1be167bcb1637a4a23cbff01a7878f0c50ee833954ea5221bb1b8c628"; _ledger.append(1)
assert _hm.digest_size == 32; _ledger.append(1)
assert _hm.block_size == 64; _ledger.append(1)
assert _hm.name == "hmac-sha256"; _ledger.append(1)

# 10) hmac.compare_digest — constant-time string compare
assert hmac.compare_digest("abc", "abc") == True; _ledger.append(1)
assert hmac.compare_digest("abc", "abd") == False; _ledger.append(1)

# 11) secrets.token_bytes — cryptographic random `bytes`
_tb = secrets.token_bytes(8)
assert type(_tb).__name__ == "bytes"; _ledger.append(1)
assert len(_tb) == 8; _ledger.append(1)

# 12) secrets.token_hex — hex-encoded random str (length 2*n)
_th = secrets.token_hex(8)
assert type(_th).__name__ == "str"; _ledger.append(1)
assert len(_th) == 16; _ledger.append(1)

# 13) secrets.token_urlsafe — url-safe random str
_tu = secrets.token_urlsafe(8)
assert type(_tu).__name__ == "str"; _ledger.append(1)
assert len(_tu) > 0; _ledger.append(1)

# 14) secrets.randbelow — uniform int in [0, n)
_rb = secrets.randbelow(100)
assert type(_rb).__name__ == "int"; _ledger.append(1)
assert _rb < 100; _ledger.append(1)
assert _rb >= 0; _ledger.append(1)

# 15) secrets.choice — uniform pick from sequence
_ch = secrets.choice([10, 20, 30])
assert type(_ch).__name__ == "int"; _ledger.append(1)
assert _ch in [10, 20, 30]; _ledger.append(1)

# 16) hasattr surface — module-level helpers
assert hasattr(keyword, "iskeyword"); _ledger.append(1)
assert hasattr(keyword, "kwlist"); _ledger.append(1)
assert hasattr(hashlib, "md5"); _ledger.append(1)
assert hasattr(hashlib, "sha256"); _ledger.append(1)
assert hasattr(hmac, "new"); _ledger.append(1)
assert hasattr(hmac, "compare_digest"); _ledger.append(1)
assert hasattr(secrets, "token_bytes"); _ledger.append(1)
assert hasattr(secrets, "randbelow"); _ledger.append(1)

# NB: hashlib.algorithms_guaranteed container type (`set` vs
# `list`), hmac.HMAC class identity, random.Random / SystemRandom
# class identity + seed-deterministic output, enum.Enum / IntEnum
# / Flag / IntFlag class identity + Enum-subclass `.name` /
# `.value` member surface + enum.auto helper all DIVERGE on
# mamba — moved to the divergence-spec fixture.

print(f"MAMBA_ASSERTION_PASS: test_keyword_hashlib_hmac_secrets_value_ops {sum(_ledger)} asserts")
