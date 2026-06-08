# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "behavior"
# case = "raw_deflate_negative_wbits_roundtrip"
# subject = "zlib.compress"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""zlib.compress: compress with wbits=-15 (raw DEFLATE, no zlib header/trailer) and decompress with wbits=-15 round-trips the input"""
import zlib

_data = b"stream test data " * 100
_raw_comp = zlib.compress(_data, wbits=-15)
_raw_decomp = zlib.decompress(_raw_comp, wbits=-15)
assert _raw_decomp == _data, "raw deflate (wbits=-15)"

print("raw_deflate_negative_wbits_roundtrip OK")
