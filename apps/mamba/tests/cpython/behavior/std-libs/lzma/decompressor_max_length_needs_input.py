# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lzma"
# dimension = "behavior"
# case = "decompressor_max_length_needs_input"
# subject = "lzma.LZMADecompressor"
# kind = "semantic"
# xfail = "LZMADecompressor is a sentinel-string stub (src/runtime/stdlib/lzma_mod.rs:87-88)"
# mem_carveout = ""
# source = "Lib/test/test_lzma.py"
# status = "filled"
# ///
"""lzma.LZMADecompressor: max_length bounds each decompress call and needs_input distinguishes buffered output from a request for more compressed data"""
import lzma


text = b"To be, or not to be, that is the question.\n" * 60
xz = lzma.compress(text, format=lzma.FORMAT_XZ)
lzd = lzma.LZMADecompressor()
piece = lzd.decompress(xz, max_length=50)
assert len(piece) == 50, "max_length caps output size"
assert lzd.needs_input is False, "more output still buffered"
rest = []
while not lzd.eof:
    rest.append(lzd.decompress(b"", max_length=4096))
assert piece + b"".join(rest) == text, "bounded reads reassemble input"
print("decompressor_max_length_needs_input OK")
