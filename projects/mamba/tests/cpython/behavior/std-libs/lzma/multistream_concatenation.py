# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lzma"
# dimension = "behavior"
# case = "multistream_concatenation"
# subject = "lzma.decompress"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_lzma.py"
# status = "filled"
# ///
"""lzma.decompress: decompress concatenates back-to-back complete streams and ignores trailing junk after a complete XZ stream"""
import lzma


text = b"To be, or not to be, that is the question.\n" * 60
xz = lzma.compress(text, format=lzma.FORMAT_XZ)
assert lzma.decompress(xz + xz) == text * 2, "multistream concatenation"
assert lzma.decompress(xz + b"garbage") == text, "trailing junk ignored"
print("multistream_concatenation OK")
