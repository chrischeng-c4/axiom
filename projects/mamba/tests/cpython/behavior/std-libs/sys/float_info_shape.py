# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "behavior"
# case = "float_info_shape"
# subject = "sys.float_info"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sys.float_info: float_info has 11 fields, radix 2, and a positive max"""
import sys

assert len(sys.float_info) == 11, f"float_info len = {len(sys.float_info)!r}"
assert sys.float_info.radix == 2, f"float_info.radix = {sys.float_info.radix!r}"
assert sys.float_info.max > 0, "float_info.max positive"
print("float_info_shape OK")
