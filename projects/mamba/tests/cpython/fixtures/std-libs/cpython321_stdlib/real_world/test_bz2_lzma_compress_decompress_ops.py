# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_bz2_lzma_compress_decompress_ops"
# subject = "cpython321.test_bz2_lzma_compress_decompress_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_bz2_lzma_compress_decompress_ops.py"
# status = "filled"
# ///
"""cpython321.test_bz2_lzma_compress_decompress_ops: execute CPython 3.12 seed test_bz2_lzma_compress_decompress_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for the `bz2` and `lzma` stdlib
# modules — the bzip2 and xz/lzma module-level codec wrappers used by
# `tarfile.open(..., mode='r:bz2')` / `tarfile.open(..., mode='r:xz')`
# / pip's wheel handling / archive tooling. Surface focuses on the
# module-level `compress(data, level)` and `decompress(data)` round
# trip — the streaming `BZ2Compressor` / `BZ2Decompressor` /
# `LZMACompressor` / `LZMADecompressor` classes are not yet covered
# by mamba so this seed sticks to the one-shot module API which both
# runtimes already agree on. Existing `test_zlib_*` covers zlib /
# gzip; `bz2` and `lzma` have no per-module fixture yet.
#
# Surface:
#   • bz2.compress(data: bytes, level: int = 9) → bytes
#       — level may be any int in 1..9 (all compress, round-trip);
#       — empty input compresses to a small non-empty header;
#       — repetitive input compresses far smaller than original;
#   • bz2.decompress(data: bytes) → bytes
#       — round-trip invariant: decompress(compress(x)) == x;
#       — agnostic to the compress level used;
#   • lzma.compress(data: bytes, preset: int = 6) → bytes
#       — preset may be any int in 1..9;
#       — round-trip on empty / short / cross-content payloads;
#   • lzma.decompress(data: bytes) → bytes
#       — round-trip invariant: decompress(compress(x)) == x;
#       — agnostic to the preset used.
import bz2
import lzma
_ledger: list[int] = []

# bz2 — round-trip across content shapes
_payloads_bz2 = [
    b"",
    b"x",
    b"hi",
    b"hello world",
    b"The quick brown fox jumps over the lazy dog.",
    b"a" * 1000,
    b"\x00\x01\x02\x03\xff\xfe\xfd",
    bytes(range(256)),
]
for _p in _payloads_bz2:
    _cmp = bz2.compress(_p)
    assert isinstance(_cmp, bytes); _ledger.append(1)
    assert bz2.decompress(_cmp) == _p; _ledger.append(1)

# bz2 — level 1..9 all round-trip
_data = b"The quick brown fox jumps over the lazy dog. " * 10
for _lv in range(1, 10):
    _cmp = bz2.compress(_data, _lv)
    assert isinstance(_cmp, bytes); _ledger.append(1)
    assert bz2.decompress(_cmp) == _data; _ledger.append(1)

# bz2 — compressing repetitive data yields a much smaller blob
_big = b"x" * 10000
_cmp_big = bz2.compress(_big)
assert len(_cmp_big) < 200; _ledger.append(1)
assert bz2.decompress(_cmp_big) == _big; _ledger.append(1)

# bz2.decompress return type discipline
assert isinstance(bz2.decompress(bz2.compress(b"abc")), bytes); _ledger.append(1)
assert isinstance(bz2.decompress(bz2.compress(b"")), bytes); _ledger.append(1)

# lzma — round-trip across content shapes
_payloads_lzma = [
    b"",
    b"x",
    b"hi",
    b"hello world",
    b"The quick brown fox jumps over the lazy dog.",
    b"a" * 1000,
    b"\x00\x01\x02\x03\xff\xfe\xfd",
    bytes(range(256)),
]
for _p in _payloads_lzma:
    _cmp = lzma.compress(_p)
    assert isinstance(_cmp, bytes); _ledger.append(1)
    assert lzma.decompress(_cmp) == _p; _ledger.append(1)

# lzma — preset 1..9 all round-trip
for _preset in range(1, 10):
    _cmp = lzma.compress(_data, preset=_preset)
    assert isinstance(_cmp, bytes); _ledger.append(1)
    assert lzma.decompress(_cmp) == _data; _ledger.append(1)

# lzma — compressing repetitive data yields a much smaller blob
_cmp_big_lzma = lzma.compress(_big)
assert len(_cmp_big_lzma) < 300; _ledger.append(1)
assert lzma.decompress(_cmp_big_lzma) == _big; _ledger.append(1)

# lzma.decompress return type discipline
assert isinstance(lzma.decompress(lzma.compress(b"abc")), bytes); _ledger.append(1)
assert isinstance(lzma.decompress(lzma.compress(b"")), bytes); _ledger.append(1)

# Cross-codec discipline: bz2 output is NOT a valid lzma blob
# (their magic bytes differ — bz2 starts with `BZh`, lzma with the
# xz magic `\xfd7zXZ`). We don't *raise* here on the cross-feed
# (mamba/cpython diverge on that — see the matching spec seed) but
# we do confirm same-codec round-trip is byte-equivalent.
_x = b"some test data for round-trip"
assert bz2.decompress(bz2.compress(_x)) == _x; _ledger.append(1)
assert lzma.decompress(lzma.compress(_x)) == _x; _ledger.append(1)

# Mixed sizes — coverage of small-block / large-block paths in the
# compressor state machines.
for _n in [0, 1, 5, 10, 50, 100, 500, 1000, 5000]:
    _payload = b"M" * _n
    assert bz2.decompress(bz2.compress(_payload)) == _payload; _ledger.append(1)
    assert lzma.decompress(lzma.compress(_payload)) == _payload; _ledger.append(1)

# Binary-payload coverage — non-printable bytes
_bin = bytes([i % 256 for i in range(500)])
assert bz2.decompress(bz2.compress(_bin)) == _bin; _ledger.append(1)
assert lzma.decompress(lzma.compress(_bin)) == _bin; _ledger.append(1)

# Multiple round-trips don't drift
_step = b"step data"
for _ in range(5):
    _step = bz2.decompress(bz2.compress(_step))
assert _step == b"step data"; _ledger.append(1)
_step = b"step data"
for _ in range(5):
    _step = lzma.decompress(lzma.compress(_step))
assert _step == b"step data"; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_bz2_lzma_compress_decompress_ops {sum(_ledger)} asserts")
