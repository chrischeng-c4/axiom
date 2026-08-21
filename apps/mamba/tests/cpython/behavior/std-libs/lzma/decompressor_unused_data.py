# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lzma"
# dimension = "behavior"
# case = "decompressor_unused_data"
# subject = "lzma.LZMADecompressor"
# kind = "semantic"
# xfail = "LZMADecompressor is a sentinel-string stub (src/runtime/stdlib/lzma_mod.rs:87-88)"
# mem_carveout = ""
# source = "Lib/test/test_lzma.py"
# status = "filled"
# ///
"""lzma.LZMADecompressor: LZMADecompressor.unused_data captures bytes trailing a complete stream"""
import lzma


data = lzma.compress(b"payload5") + b"TRAILING"
decomp = lzma.LZMADecompressor()
r = decomp.decompress(data)
assert r == b"payload5", f"payload = {r!r}"
assert decomp.unused_data == b"TRAILING", f"unused = {decomp.unused_data!r}"
print("decompressor_unused_data OK")
