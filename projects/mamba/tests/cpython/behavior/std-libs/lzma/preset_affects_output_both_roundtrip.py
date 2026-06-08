# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lzma"
# dimension = "behavior"
# case = "preset_affects_output_both_roundtrip"
# subject = "lzma.compress"
# kind = "semantic"
# xfail = "lzma.compress ignores the preset kwarg (data-only shim, src/runtime/stdlib/lzma_mod.rs:153)"
# mem_carveout = ""
# source = "Lib/test/test_lzma.py"
# status = "filled"
# ///
"""lzma.compress: compress at preset 0 and preset 9 both decompress back to the original data"""
import lzma


data = b"repetitive " * 500
c0 = lzma.compress(data, preset=0)
c9 = lzma.compress(data, preset=9)
assert lzma.decompress(c0) == data, "preset 0 decompresses"
assert lzma.decompress(c9) == data, "preset 9 decompresses"
print("preset_affects_output_both_roundtrip OK")
