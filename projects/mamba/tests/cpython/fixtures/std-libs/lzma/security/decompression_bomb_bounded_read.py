# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lzma"
# dimension = "security"
# case = "decompression_bomb_bounded_read"
# subject = "lzma.LZMADecompressor"
# kind = "semantic"
# xfail = "lzma.LZMADecompressor is a sentinel-string stub; constructing it raises and bounded decompress(max_length=...) is unimplemented (src/runtime/stdlib/lzma_mod.rs:87-88)"
# mem_carveout = ""
# source = "Lib/test/test_lzma.py"
# status = "filled"
# ///
"""lzma.LZMADecompressor: a high-ratio decompression bomb is defused by a bounded LZMADecompressor: max_length caps each call so output stays under a fixed cap and the full uncompressed payload is never materialized"""
import lzma


# A high-ratio "bomb": 32 MiB of a single repeated byte compresses to a tiny
# .xz blob. A naive decode would balloon to 32 MiB; a bounded read refuses to.
UNCOMPRESSED = 32 * 1024 * 1024
PAYLOAD = b"\x00" * UNCOMPRESSED
BLOB = lzma.compress(PAYLOAD, format=lzma.FORMAT_XZ)

ratio = UNCOMPRESSED / len(BLOB)
assert ratio > 1000, f"blob is genuinely high-ratio: {ratio:.0f}x"

# Defensive decode: pull at most CHUNK bytes per call via max_length and refuse
# to exceed an OUTPUT_CAP. The bound, not the blob, decides how much we read.
CHUNK = 64 * 1024
OUTPUT_CAP = 8 * 1024 * 1024

dec = lzma.LZMADecompressor()
total = 0
capped = False
fed = False
while not dec.eof:
    if dec.needs_input and not fed:
        piece = dec.decompress(BLOB, max_length=CHUNK)
        fed = True
    else:
        piece = dec.decompress(b"", max_length=CHUNK)
    assert len(piece) <= CHUNK, f"max_length bound violated: {len(piece)} > {CHUNK}"
    total += len(piece)
    if total >= OUTPUT_CAP:
        capped = True
        break

assert capped, "expected to hit the output cap before draining the whole bomb"
assert total < UNCOMPRESSED, "we never materialized the full uncompressed payload"
assert not dec.eof, "decompressor still has buffered data; stream not exhausted"

# A tiny per-call bound is also honored exactly (cap is per-request).
dec2 = lzma.LZMADecompressor()
first = dec2.decompress(BLOB, max_length=10)
assert len(first) == 10, f"small max_length honored exactly: {len(first)}"
assert dec2.needs_input is False, "output still buffered after a capped read"
print("decompression_bomb_bounded_read OK")
