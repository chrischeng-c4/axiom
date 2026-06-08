# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bz2"
# dimension = "behavior"
# case = "compress_decompress_roundtrip"
# subject = "bz2.compress"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bz2.py"
# status = "filled"
# ///
"""bz2.compress: compress then decompress preserves data exactly and shrinks repetitive input"""
import bz2

data = b"The quick brown fox jumps over the lazy dog\n" * 50
compressed = bz2.compress(data)
assert bz2.decompress(compressed) == data, "compress/decompress round-trip"
assert len(compressed) < len(data), "compression reduces size for repetitive data"
print("compress_decompress_roundtrip OK")
