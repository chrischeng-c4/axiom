# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gzip"
# dimension = "behavior"
# case = "higher_level_smaller_or_equal_same_output"
# subject = "gzip.compress"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""gzip.compress: compresslevel 1 and 9 both decompress back to the same input, and for repetitive data level 9 produces output no larger than level 1"""
import gzip

_data = b"aaaa" * 1000
_c1 = gzip.compress(_data, compresslevel=1)
_c9 = gzip.compress(_data, compresslevel=9)
assert gzip.decompress(_c1) == _data, "level 1 correct"
assert gzip.decompress(_c9) == _data, "level 9 correct"
# Higher compression should produce smaller or equal output for repetitive data.
assert len(_c9) <= len(_c1), f"level 9 <= level 1: {len(_c9)} vs {len(_c1)}"

print("higher_level_smaller_or_equal_same_output OK")
