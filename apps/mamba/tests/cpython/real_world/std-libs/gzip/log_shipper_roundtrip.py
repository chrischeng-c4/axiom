# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gzip"
# dimension = "real_world"
# case = "log_shipper_roundtrip"
# subject = "gzip.compress"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_gzip.py"
# status = "filled"
# ///
"""gzip.compress: a log-shipper batches plaintext log lines, gzip-compresses each batch, the consumer decompresses and CRC32-validates: two distinct batches compress to distinct streams, each round-trips exactly, the gzip magic/DEFLATE header is present, zlib.crc32 over decoded equals crc over original, and the typeshed-declared names all resolve"""
import gzip
import zlib

# Two distinct synthetic log batches — enough variety that any
# off-by-one or buffer-reuse bug would show up.
BATCH_A = b"\n".join([
    b"2026-05-13T18:30:00Z INFO app=ingest req=abc123 status=200 size=4096",
    b"2026-05-13T18:30:01Z INFO app=ingest req=abc124 status=200 size=8192",
    b"2026-05-13T18:30:02Z WARN app=ingest req=abc125 status=429 retry=1",
    b"2026-05-13T18:30:03Z INFO app=ingest req=abc126 status=200 size=512",
    b"2026-05-13T18:30:04Z ERROR app=ingest req=abc127 status=500 trace=...",
])

BATCH_B = b"\n".join([
    b"unrelated content with binary tail follows:",
    # Avoid bytes(reversed(range(N))) — mamba currently overflows the
    # reverse-iterator length to 2^32. Use a bytes literal directly so
    # the fixture's inputs are byte-identical across runtimes.
    bytes(range(64)) + bytes([63 - i for i in range(64)]),
    b"more text after the binary payload",
])


def round_trip(batch: bytes, label: str) -> None:
    compressed = gzip.compress(batch)
    # gzip magic bytes are 0x1F 0x8B at offsets 0 and 1.
    assert compressed[0] == 0x1F, f"{label}: missing gzip magic byte 0, got {compressed[0]:#x}"
    assert compressed[1] == 0x8B, f"{label}: missing gzip magic byte 1, got {compressed[1]:#x}"
    # CM byte (compression method) = 8 means DEFLATE.
    assert compressed[2] == 0x08, f"{label}: CM byte should be DEFLATE=8, got {compressed[2]:#x}"

    decoded = gzip.decompress(compressed)
    assert decoded == batch, f"{label}: round-trip mismatch, len(decoded)={len(decoded)} vs len(batch)={len(batch)}"

    crc_decoded = zlib.crc32(decoded)
    crc_orig = zlib.crc32(batch)
    assert crc_decoded == crc_orig, f"{label}: CRC32 mismatch decoded={crc_decoded:#x} orig={crc_orig:#x}"


round_trip(BATCH_A, "batch_a")
round_trip(BATCH_B, "batch_b")

# Verify the two compressed streams are distinct — if a buggy shim
# returned a single cached buffer the streams would match byte-for-byte.
c_a = gzip.compress(BATCH_A)
c_b = gzip.compress(BATCH_B)
assert c_a != c_b, "distinct inputs produced identical compressed streams"

# Surface-resolve check for the typeshed-declared names — they must be
# present on the module so downstream consumers don't AttributeError.
assert hasattr(gzip, "GzipFile"), "gzip.GzipFile not present"
assert hasattr(gzip, "BadGzipFile"), "gzip.BadGzipFile not present"
assert hasattr(gzip, "open"), "gzip.open not present"
assert hasattr(gzip, "compress"), "gzip.compress not present"
assert hasattr(gzip, "decompress"), "gzip.decompress not present"

print("log_shipper_roundtrip OK")
