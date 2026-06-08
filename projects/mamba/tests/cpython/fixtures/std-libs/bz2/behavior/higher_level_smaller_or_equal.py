# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bz2"
# dimension = "behavior"
# case = "higher_level_smaller_or_equal"
# subject = "bz2.compress"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bz2.py"
# status = "filled"
# ///
"""bz2.compress: compresslevel=9 output is no larger than level=1 and still decompresses correctly"""
import bz2

data = b"abcdefghij" * 200
c1 = bz2.compress(data, compresslevel=1)
c9 = bz2.compress(data, compresslevel=9)
assert len(c9) <= len(c1), f"level 9 <= level 1: {len(c9)} vs {len(c1)}"
assert bz2.decompress(c9) == data, "level 9 decompresses correctly"
print("higher_level_smaller_or_equal OK")
