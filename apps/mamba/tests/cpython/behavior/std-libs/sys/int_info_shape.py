# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "behavior"
# case = "int_info_shape"
# subject = "sys.int_info"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sys.int_info: int_info has 4 fields, bits_per_digit a multiple of 5, sizeof_digit >= 1, and default_max_str_digits above the str_digits_check_threshold"""
import sys

assert len(sys.int_info) == 4, f"int_info len = {len(sys.int_info)!r}"
assert sys.int_info.bits_per_digit % 5 == 0, \
    f"bits_per_digit = {sys.int_info.bits_per_digit!r}"
assert sys.int_info.sizeof_digit >= 1, "sizeof_digit >= 1"
assert sys.int_info.default_max_str_digits > sys.int_info.str_digits_check_threshold, \
    "default_max_str_digits exceeds the check threshold"
print("int_info_shape OK")
