# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lzma"
# dimension = "behavior"
# case = "incremental_compress_decompress"
# subject = "lzma.LZMACompressor"
# kind = "semantic"
# xfail = "LZMACompressor/LZMADecompressor are sentinel-string stubs (src/runtime/stdlib/lzma_mod.rs:85-88)"
# mem_carveout = ""
# source = "Lib/test/test_lzma.py"
# status = "filled"
# ///
"""lzma.LZMACompressor: LZMACompressor fed in parts then flushed produces a stream that LZMADecompressor reassembles exactly, with eof True after"""
import lzma


comp = lzma.LZMACompressor()
parts = [b"part1 ", b"part2 ", b"part3"]
compressed = b"".join([comp.compress(p) for p in parts]) + comp.flush()
decomp = lzma.LZMADecompressor()
result = decomp.decompress(compressed)
assert result == b"part1 part2 part3", f"incremental = {result!r}"
assert decomp.eof is True, "eof after complete decompression"
print("incremental_compress_decompress OK")
