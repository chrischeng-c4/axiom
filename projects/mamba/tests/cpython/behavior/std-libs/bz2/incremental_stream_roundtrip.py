# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bz2"
# dimension = "behavior"
# case = "incremental_stream_roundtrip"
# subject = "bz2.BZ2Compressor"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bz2.py"
# status = "filled"
# ///
"""bz2.BZ2Compressor: BZ2Compressor.compress chunks + flush concatenate into a stream BZ2Decompressor reassembles, setting eof"""
import bz2

comp = bz2.BZ2Compressor()
parts = [b"chunk1:", b"chunk2:", b"chunk3"]
compressed = b"".join([comp.compress(p) for p in parts]) + comp.flush()
decomp = bz2.BZ2Decompressor()
result = decomp.decompress(compressed)
assert result == b"chunk1:chunk2:chunk3", f"incremental = {result!r}"
assert decomp.eof is True, "eof set after complete decompression"
print("incremental_stream_roundtrip OK")
