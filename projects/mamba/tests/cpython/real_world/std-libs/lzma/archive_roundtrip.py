# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lzma"
# dimension = "real_world"
# case = "archive_roundtrip"
# subject = "lzma.compress"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_lzma.py"
# status = "filled"
# ///
"""lzma.compress: an archival pipeline LZMA-compresses two distinct document batches, verifies each round-trips, checks the .xz magic header, and cross-checks zlib.crc32 over decoded vs original bytes"""
import lzma


import zlib

BATCH_A = b"\n".join([
    b"2026-05-13T18:30:00Z record=ingest doc=abc123 type=invoice size=4096",
    b"2026-05-13T18:30:01Z record=ingest doc=abc124 type=invoice size=8192",
    b"2026-05-13T18:30:02Z record=update doc=abc125 type=receipt rev=1",
    b"2026-05-13T18:30:03Z record=ingest doc=abc126 type=invoice size=512",
    b"2026-05-13T18:30:04Z record=delete doc=abc127 type=receipt rev=2",
])

BATCH_B = b"\n".join([
    b"unrelated archive content with binary tail follows:",
    bytes(range(64)) + bytes([63 - i for i in range(64)]),
    b"more text after the binary payload",
])


def round_trip(batch, label):
    compressed = lzma.compress(batch)
    # .xz stream header: 6 bytes 0xFD '7' 'z' 'X' 'Z' 0x00.
    assert compressed[:6] == b"\xfd7zXZ\x00", f"{label}: xz magic {compressed[:6]!r}"
    decoded = lzma.decompress(compressed)
    assert decoded == batch, f"{label}: round-trip mismatch"
    assert zlib.crc32(decoded) == zlib.crc32(batch), f"{label}: CRC32 mismatch"
    print(f"{label}: orig={len(batch)}B compressed={len(compressed)}B crc={zlib.crc32(batch):#x}")


round_trip(BATCH_A, "batch_a")
round_trip(BATCH_B, "batch_b")

# Distinct inputs must produce distinct compressed streams (no cached buffer).
assert lzma.compress(BATCH_A) != lzma.compress(BATCH_B), "distinct inputs -> distinct streams"

# Surface names resolve and int constants equal their CPython values.
for name in ("compress", "decompress", "LZMAError", "LZMAFile",
             "LZMACompressor", "LZMADecompressor", "open"):
    assert hasattr(lzma, name), f"lzma.{name} not present"
assert lzma.FORMAT_AUTO == 0 and lzma.FORMAT_XZ == 1, "FORMAT_AUTO/XZ"
assert lzma.CHECK_CRC32 == 1 and lzma.CHECK_CRC64 == 4, "CHECK_CRC32/CRC64"
print("archive_roundtrip OK")
