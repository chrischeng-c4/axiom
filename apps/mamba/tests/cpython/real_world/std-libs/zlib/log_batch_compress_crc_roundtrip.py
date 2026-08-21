# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "real_world"
# case = "log_batch_compress_crc_roundtrip"
# subject = "zlib.compress"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""zlib.compress: a log shipper compresses each plaintext batch with zlib.compress, the consumer decompresses and crc32-validates: two distinct batches compress to distinct streams, each round-trips exactly, and zlib.crc32 over the decoded payload equals the crc over the original"""
import zlib

# Producer side: batch plaintext log lines into newline-delimited payloads.
_batch_a = "\n".join(f"2026-05-29T10:00:{n:02d} INFO request {n} served" for n in range(60)).encode()
_batch_b = "\n".join(f"2026-05-29T10:01:{n:02d} WARN retry {n} backoff" for n in range(60)).encode()

# Each batch is compressed with its CRC32 captured before transport.
_packets = []
for _payload in (_batch_a, _batch_b):
    _packets.append((zlib.compress(_payload, level=6), zlib.crc32(_payload), len(_payload)))

# Distinct batches yield distinct compressed streams.
assert _packets[0][0] != _packets[1][0], "distinct batches compress distinctly"

# Consumer side: decompress, then CRC32-validate against the transported checksum.
_recovered = []
for _stream, _crc, _size in _packets:
    _decoded = zlib.decompress(_stream)
    assert zlib.crc32(_decoded) == _crc, "crc32 over decoded matches transported crc"
    assert len(_decoded) == _size, "decoded length matches original"
    _recovered.append(_decoded)

assert _recovered[0] == _batch_a, "batch A round-trips exactly"
assert _recovered[1] == _batch_b, "batch B round-trips exactly"

# Compression actually shrinks the repetitive log text.
assert len(_packets[0][0]) < len(_batch_a), "compressed batch A smaller than plaintext"

print("log_batch_compress_crc_roundtrip OK")
