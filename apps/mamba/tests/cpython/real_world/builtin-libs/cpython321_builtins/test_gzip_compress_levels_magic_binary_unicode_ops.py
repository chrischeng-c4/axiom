# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "cpython321_builtins"
# dimension = "real_world"
# case = "test_gzip_compress_levels_magic_binary_unicode_ops"
# subject = "cpython321.test_gzip_compress_levels_magic_binary_unicode_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_gzip_compress_levels_magic_binary_unicode_ops.py"
# status = "filled"
# ///
"""cpython321.test_gzip_compress_levels_magic_binary_unicode_ops: execute CPython 3.12 seed test_gzip_compress_levels_magic_binary_unicode_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Atomic 181: gzip compression levels + RFC1952 magic bytes + binary/unicode roundtrip
#
# Covers `gzip` surfaces beyond the two-assert minimal roundtrip in
# test_zlib_gzip_ops.py (gzip.compress/decompress empty + non-empty):
#   - gzip.compress with explicit compression levels (1, 9) round-trip
#   - RFC1952 magic-byte header (first two bytes == 0x1f 0x8b) on
#     non-empty payloads
#   - large repeated payload compresses smaller than input
#   - binary-safe roundtrip across raw \x00 / \xff / mixed-byte payload
#   - UTF-8 multibyte payload round-trip (Latin-1 + CJK)
import gzip

_ledger = []

# --- baseline compress/decompress ---
data = b"hello world hello world hello world"
c = gzip.compress(data)
assert isinstance(c, bytes); _ledger.append(1)
assert gzip.decompress(c) == data; _ledger.append(1)
assert gzip.decompress(gzip.compress(b"")) == b""; _ledger.append(1)

# --- compression level 1 (fastest) round-trip ---
c1 = gzip.compress(data, 1)
assert isinstance(c1, bytes); _ledger.append(1)
assert gzip.decompress(c1) == data; _ledger.append(1)

# --- compression level 9 (best) round-trip ---
c9 = gzip.compress(data, 9)
assert isinstance(c9, bytes); _ledger.append(1)
assert gzip.decompress(c9) == data; _ledger.append(1)

# --- RFC1952 magic bytes (gzip file header) ---
assert c[:2] == b"\x1f\x8b"; _ledger.append(1)
assert c1[:2] == b"\x1f\x8b"; _ledger.append(1)
assert c9[:2] == b"\x1f\x8b"; _ledger.append(1)

# --- large repeated payload compresses smaller ---
big = b"x" * 1000
assert len(gzip.compress(big)) < 1000; _ledger.append(1)
assert gzip.decompress(gzip.compress(big)) == big; _ledger.append(1)

# --- binary-safe roundtrip ---
assert gzip.decompress(gzip.compress(b"\x00")) == b"\x00"; _ledger.append(1)
assert gzip.decompress(gzip.compress(b"\xff")) == b"\xff"; _ledger.append(1)
assert gzip.decompress(gzip.compress(b"\x00\x01\x02\x03")) == b"\x00\x01\x02\x03"; _ledger.append(1)
assert gzip.decompress(gzip.compress(b"\xfc\xfd\xfe\xff")) == b"\xfc\xfd\xfe\xff"; _ledger.append(1)

# --- mixed-byte non-trivial payload ---
mixed = b"\x00\x01\x02\x03\xfd\xfe\xff" * 50
assert gzip.decompress(gzip.compress(mixed)) == mixed; _ledger.append(1)

# --- UTF-8 multibyte payload (Latin-1 + CJK) ---
unicode_bytes = "héllo wörld 你好".encode("utf-8")
assert gzip.decompress(gzip.compress(unicode_bytes)) == unicode_bytes; _ledger.append(1)
assert gzip.decompress(gzip.compress(unicode_bytes)).decode("utf-8") == "héllo wörld 你好"; _ledger.append(1)

# --- single byte / two byte payloads ---
assert gzip.decompress(gzip.compress(b"a")) == b"a"; _ledger.append(1)
assert gzip.decompress(gzip.compress(b"ab")) == b"ab"; _ledger.append(1)

# --- level-bracket round-trip on UTF-8 payload ---
assert gzip.decompress(gzip.compress(unicode_bytes, 1)) == unicode_bytes; _ledger.append(1)
assert gzip.decompress(gzip.compress(unicode_bytes, 9)) == unicode_bytes; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_gzip_compress_levels_magic_binary_unicode_ops {sum(_ledger)} asserts")
