# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lzma"
# dimension = "behavior"
# case = "decompressor_check_attr"
# subject = "lzma.LZMADecompressor"
# kind = "semantic"
# xfail = "LZMADecompressor is a sentinel-string stub (src/runtime/stdlib/lzma_mod.rs:87-88)"
# mem_carveout = ""
# source = "Lib/test/test_lzma.py"
# status = "filled"
# ///
"""lzma.LZMADecompressor: after decoding an XZ stream the decompressor's .check reports CHECK_CRC64 (the default integrity check)"""
import lzma


text = b"To be, or not to be, that is the question.\n" * 60
xz = lzma.compress(text, format=lzma.FORMAT_XZ)
lzd = lzma.LZMADecompressor()
lzd.decompress(xz)
assert lzd.check == lzma.CHECK_CRC64, f"check = {lzd.check!r}"
print("decompressor_check_attr OK")
