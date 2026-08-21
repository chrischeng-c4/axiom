import hashlib

_ledger: list[int] = []

# sha256 fixed vector
assert hashlib.sha256(b"abc").hexdigest() == (
    "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
), "sha256('abc') matches NIST vector"
_ledger.append(1)

# md5 fixed vector
assert hashlib.md5(b"abc").hexdigest() == "900150983cd24fb0d6963f7d28e17f72", "md5('abc')"
_ledger.append(1)

# sha1 fixed vector
assert hashlib.sha1(b"abc").hexdigest() == "a9993e364706816aba3e25717850c26c9cd0d89d", "sha1('abc')"
_ledger.append(1)

# sha512 fixed vector (length 128 hex chars)
sha512_abc = hashlib.sha512(b"abc").hexdigest()
assert len(sha512_abc) == 128, "sha512 hex length"
_ledger.append(1)

assert sha512_abc.startswith("ddaf35a193617aba"), "sha512('abc') starts with NIST prefix"
_ledger.append(1)

# sha3_256 fixed vector
assert hashlib.sha3_256(b"abc").hexdigest() == (
    "3a985da74fe225b2045c172d6bd390bd855f086e3e9d525b46bfe24511431532"
), "sha3_256('abc') matches NIST vector"
_ledger.append(1)

# digest_size / block_size
h = hashlib.sha256()
assert h.digest_size == 32, "sha256 digest_size is 32 bytes"
_ledger.append(1)

assert h.block_size == 64, "sha256 block_size is 64 bytes"
_ledger.append(1)

# incremental update is equivalent to one-shot
inc = hashlib.sha256()
inc.update(b"ab")
inc.update(b"c")
assert inc.hexdigest() == hashlib.sha256(b"abc").hexdigest(), "incremental update == one-shot"
_ledger.append(1)

# hashlib.new(name, data)
assert hashlib.new("sha256", b"abc").hexdigest() == (
    "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
), "hashlib.new('sha256') round-trips"
_ledger.append(1)

# algorithms_available / algorithms_guaranteed include sha256
assert "sha256" in hashlib.algorithms_available, "sha256 in algorithms_available"
_ledger.append(1)

assert "sha256" in hashlib.algorithms_guaranteed, "sha256 in algorithms_guaranteed"
_ledger.append(1)

# digest() returns bytes of digest_size
d_bytes = hashlib.sha256(b"abc").digest()
assert isinstance(d_bytes, bytes), "digest() returns bytes"
_ledger.append(1)

assert len(d_bytes) == 32, "sha256 digest() is 32 bytes"
_ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_hashlib {sum(_ledger)} asserts")
