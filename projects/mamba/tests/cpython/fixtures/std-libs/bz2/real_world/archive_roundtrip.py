# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bz2"
# dimension = "real_world"
# case = "archive_roundtrip"
# subject = "bz2.compress"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bz2.py"
# status = "filled"
# ///
"""bz2.compress: an archival pipeline bzip2-compresses two distinct document batches, verifies the BZh magic header, round-trips each batch, and cross-checks zlib.crc32 over the decoded bytes"""
import bz2
import zlib

# Two distinct synthetic archive batches — enough variety that any
# off-by-one or buffer-reuse bug would surface.
BATCH_A = b"\n".join([
    b"2026-05-15T18:30:00Z record=ingest doc=abc123 type=invoice size=4096",
    b"2026-05-15T18:30:01Z record=ingest doc=abc124 type=invoice size=8192",
    b"2026-05-15T18:30:02Z record=update doc=abc125 type=receipt rev=1",
    b"2026-05-15T18:30:03Z record=ingest doc=abc126 type=invoice size=512",
    b"2026-05-15T18:30:04Z record=delete doc=abc127 type=receipt rev=2",
])

BATCH_B = b"\n".join([
    b"unrelated archive content with binary tail follows:",
    # Avoid bytes(reversed(range(N))) — mamba currently overflows the
    # reverse-iterator length to 2^32. Use a bytes literal-equivalent
    # built via list-comp instead.
    bytes(range(64)) + bytes([63 - i for i in range(64)]),
    b"more text after the binary payload",
])


def round_trip(batch: bytes, label: str) -> None:
    compressed = bz2.compress(batch)
    # bzip2 stream header: "BZh" + 1-byte block-size digit '1'..'9'.
    assert compressed[0] == ord('B'), f"{label}: bz2 magic byte 0, got {compressed[0]:#x}"
    assert compressed[1] == ord('Z'), f"{label}: bz2 magic byte 1, got {compressed[1]:#x}"
    assert compressed[2] == ord('h'), f"{label}: bz2 magic byte 2, got {compressed[2]:#x}"
    assert ord('1') <= compressed[3] <= ord('9'), \
        f"{label}: block-size digit out of range, got {compressed[3]:#x}"

    decoded = bz2.decompress(compressed)
    assert decoded == batch, \
        f"{label}: round-trip mismatch, len(decoded)={len(decoded)} vs len(batch)={len(batch)}"

    crc_decoded = zlib.crc32(decoded)
    crc_orig = zlib.crc32(batch)
    assert crc_decoded == crc_orig, \
        f"{label}: CRC32 mismatch decoded={crc_decoded:#x} orig={crc_orig:#x}"


round_trip(BATCH_A, "batch_a")
round_trip(BATCH_B, "batch_b")

# Verify the two compressed streams are distinct — if a buggy shim
# returned a single cached buffer the streams would match byte-for-byte.
c_a = bz2.compress(BATCH_A)
c_b = bz2.compress(BATCH_B)
assert c_a != c_b, "distinct inputs produced identical compressed streams"

# Surface-resolve check for the streaming attributes — they must be
# present so bz2.BZ2File etc. don't AttributeError under conformance tests.
for name in ("compress", "decompress", "BZ2File",
             "BZ2Compressor", "BZ2Decompressor", "open"):
    assert hasattr(bz2, name), f"bz2.{name} not present"

print("archive_roundtrip OK")
