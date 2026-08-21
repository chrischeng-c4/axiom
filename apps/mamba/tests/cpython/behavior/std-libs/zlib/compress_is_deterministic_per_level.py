# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "behavior"
# case = "compress_is_deterministic_per_level"
# subject = "zlib.compress"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""zlib.compress: two compress calls on the same data at the same level produce byte-identical streams"""
import zlib

assert zlib.compress(b"test", level=6) == zlib.compress(b"test", level=6), "deterministic"
_data = b"deterministic payload " * 64
assert zlib.compress(_data, level=9) == zlib.compress(_data, level=9), "deterministic level 9"

print("compress_is_deterministic_per_level OK")
