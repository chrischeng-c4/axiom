# Operational AssertionPass seed for hmac surfaces beyond
# test_secrets_hmac_ops (which only checks SHA256("hello") digest
# length). Surface: hmac.new(key, msg, algo).hexdigest() and .digest()
# for the four common hash algorithms md5 / sha1 / sha256 / sha512,
# each producing the expected hex-string and bytes lengths; the
# digest is deterministic — same key+msg+algo gives the same output;
# a different key OR a different message produces a different digest;
# the canonical SHA256 test vector for key="key" /
# msg="The quick brown fox jumps over the lazy dog"; .digest() returns
# bytes (not str); hmac.compare_digest performs constant-time
# equality on both str and bytes operands; .update(...) accumulates
# message bytes such that two update() calls give the same digest as
# passing the concatenated message at construction time.
import hmac
_ledger: list[int] = []

# SHA256 — 64-char hex digest
h = hmac.new(b"secret", b"message", "sha256")
d = h.hexdigest()
assert len(d) == 64; _ledger.append(1)
# Every char is a lowercase hex digit
hex_chars = "0123456789abcdef"
assert all(c in hex_chars for c in d); _ledger.append(1)

# Deterministic — same input always produces the same digest
h2 = hmac.new(b"secret", b"message", "sha256")
assert h.hexdigest() == h2.hexdigest(); _ledger.append(1)

# Different key → different digest
h3 = hmac.new(b"other", b"message", "sha256")
assert h.hexdigest() != h3.hexdigest(); _ledger.append(1)

# Different message → different digest
h4 = hmac.new(b"secret", b"different", "sha256")
assert h.hexdigest() != h4.hexdigest(); _ledger.append(1)

# Canonical RFC test vector — HMAC-SHA256(key="key",
# msg="The quick brown fox jumps over the lazy dog")
v = hmac.new(b"key", b"The quick brown fox jumps over the lazy dog", "sha256")
expected = "f7bc83f430538424b13298e6aa6fb143ef4d59a14946175997479dbc2d1a3cd8"
assert v.hexdigest() == expected; _ledger.append(1)

# .digest() returns raw bytes (not str)
h5 = hmac.new(b"k", b"m", "sha256")
assert type(h5.digest()).__name__ == "bytes"; _ledger.append(1)
# SHA256 digest is 32 bytes (= 256 bits / 8)
assert len(h5.digest()) == 32; _ledger.append(1)

# MD5 — 32-char hex / 16-byte digest
hmd5 = hmac.new(b"k", b"m", "md5")
assert len(hmd5.hexdigest()) == 32; _ledger.append(1)
assert len(hmd5.digest()) == 16; _ledger.append(1)

# SHA1 — 40-char hex / 20-byte digest
hsha1 = hmac.new(b"k", b"m", "sha1")
assert len(hsha1.hexdigest()) == 40; _ledger.append(1)
assert len(hsha1.digest()) == 20; _ledger.append(1)

# SHA512 — 128-char hex / 64-byte digest
hsha512 = hmac.new(b"k", b"m", "sha512")
assert len(hsha512.hexdigest()) == 128; _ledger.append(1)
assert len(hsha512.digest()) == 64; _ledger.append(1)

# Different hash algorithms on the same key+msg → different digests
sha256_d = hmac.new(b"k", b"m", "sha256").hexdigest()
sha512_d = hmac.new(b"k", b"m", "sha512").hexdigest()
md5_d = hmac.new(b"k", b"m", "md5").hexdigest()
assert sha256_d != sha512_d; _ledger.append(1)
assert sha256_d != md5_d; _ledger.append(1)
assert sha512_d != md5_d; _ledger.append(1)

# hmac.compare_digest — equal strings return True, unequal False
assert hmac.compare_digest("abc", "abc") == True; _ledger.append(1)
assert hmac.compare_digest("abc", "abd") == False; _ledger.append(1)
assert hmac.compare_digest("", "") == True; _ledger.append(1)
# bytes operands are also accepted
assert hmac.compare_digest(b"abc", b"abc") == True; _ledger.append(1)
assert hmac.compare_digest(b"abc", b"abd") == False; _ledger.append(1)

# Chained .update(...) calls — two updates equal one larger message
hu = hmac.new(b"k", digestmod="sha256")
hu.update(b"hello")
hu.update(b" world")
hone = hmac.new(b"k", b"hello world", "sha256")
assert hu.hexdigest() == hone.hexdigest(); _ledger.append(1)

# Three-piece chained update equals one large message
ht = hmac.new(b"k", digestmod="sha256")
ht.update(b"a")
ht.update(b"b")
ht.update(b"c")
hone2 = hmac.new(b"k", b"abc", "sha256")
assert ht.hexdigest() == hone2.hexdigest(); _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: test_hmac_digest_ops {sum(_ledger)} asserts")
