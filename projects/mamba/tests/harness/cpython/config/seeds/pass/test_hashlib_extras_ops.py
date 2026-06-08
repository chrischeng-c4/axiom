# Operational AssertionPass seed for hashlib surfaces beyond
# test_hashlib_ops (which covers md5/sha1/sha256 hexdigest only).
# Surface: sha224 / sha384 / sha512 hexdigest of canonical "abc",
# incremental .update() over multiple chunks, .digest() bytes prefix,
# .name attribute readback, .digest_size attribute, hashlib.new(name,
# data) factory.
import hashlib
_ledger: list[int] = []

# sha224 hexdigest of "abc"
assert hashlib.sha224(b"abc").hexdigest() == "23097d223405d8228642a477bda255b32aadbce4bda0b3f7e36c9da7"; _ledger.append(1)

# sha384 hexdigest of "abc"
assert hashlib.sha384(b"abc").hexdigest() == "cb00753f45a35e8bb5a03d699ac65007272c32ab0eded1631a8b605a43ff5bed8086072ba1e7cc2358baeca134c825a7"; _ledger.append(1)

# sha512 hexdigest of "abc"
assert hashlib.sha512(b"abc").hexdigest() == "ddaf35a193617abacc417349ae20413112e6fa4e89a97ea20a9eeee64b55d39a2192992a274fc1a836ba3c23a3feebbd454d4423643ce80e2a9ac94fa54ca49f"; _ledger.append(1)

# Incremental .update() concatenates chunks before finalizing —
# update(b"ab") then update(b"c") must equal hashing b"abc" in one shot
h = hashlib.sha256()
h.update(b"ab")
h.update(b"c")
assert h.hexdigest() == hashlib.sha256(b"abc").hexdigest(); _ledger.append(1)

# digest() returns the raw bytes; the first 4 bytes of sha256("abc")
# are 0xba 0x78 0x16 0xbf
assert hashlib.sha256(b"abc").digest()[:4] == b"\xba\x78\x16\xbf"; _ledger.append(1)

# .name attribute identifies the algorithm
assert hashlib.sha256().name == "sha256"; _ledger.append(1)
assert hashlib.sha512().name == "sha512"; _ledger.append(1)

# .digest_size is the byte length of the raw digest
assert hashlib.sha256().digest_size == 32; _ledger.append(1)
assert hashlib.sha512().digest_size == 64; _ledger.append(1)

# hashlib.new(name, data) is equivalent to picking the constructor by name
expected = hashlib.sha256(b"abc").hexdigest()
assert hashlib.new("sha256", b"abc").hexdigest() == expected; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: test_hashlib_extras_ops {sum(_ledger)} asserts")
