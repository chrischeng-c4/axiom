# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lzma"
# dimension = "behavior"
# case = "compress_decompress_roundtrip"
# subject = "lzma.compress"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_lzma.py"
# status = "filled"
# ///
"""lzma.compress: compress then decompress restores the original bytes exactly and the compressed form is smaller for repetitive data"""
import lzma


data = b"The quick brown fox jumps over the lazy dog\n" * 30
c = lzma.compress(data)
assert lzma.decompress(c) == data, "compress/decompress round-trip"
assert len(c) < len(data), "compression reduces size for repetitive data"
print("compress_decompress_roundtrip OK")
