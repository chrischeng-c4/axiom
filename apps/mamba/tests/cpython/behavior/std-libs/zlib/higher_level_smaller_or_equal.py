# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "behavior"
# case = "higher_level_smaller_or_equal"
# subject = "zlib.compress"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""zlib.compress: for repetitive data level 9 produces a stream no larger than level 1, and both decompress back to the original"""
import zlib

_compressible = b"abc" * 5000
_c1 = zlib.compress(_compressible, level=1)
_c9 = zlib.compress(_compressible, level=9)
assert len(_c9) <= len(_c1), f"level 9 ({len(_c9)}) <= level 1 ({len(_c1)})"
assert zlib.decompress(_c1) == _compressible, "level 1 round-trip"
assert zlib.decompress(_c9) == _compressible, "level 9 round-trip"

print("higher_level_smaller_or_equal OK")
