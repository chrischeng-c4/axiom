# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lzma"
# dimension = "behavior"
# case = "decompressor_reuse_after_error"
# subject = "lzma.LZMADecompressor"
# kind = "semantic"
# xfail = "LZMADecompressor is a sentinel-string stub (src/runtime/stdlib/lzma_mod.rs:87-88)"
# mem_carveout = ""
# source = "Lib/test/test_lzma.py"
# status = "filled"
# ///
"""lzma.LZMADecompressor: bug 28275: a decompressor that already raised on invalid input keeps raising on reuse"""
import lzma


lzd = lzma.LZMADecompressor()
raised_each_time = True
for _attempt in range(2):
    try:
        lzd.decompress(b"this is not a valid lzma stream")
        raised_each_time = False
        break
    except lzma.LZMAError:
        pass
assert raised_each_time, "bug 28275: decompressor keeps raising on reuse"
print("decompressor_reuse_after_error OK")
