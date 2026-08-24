# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "format_map"
# dimension = "behavior"
# case = "format_map_formats_fraction_float_spec"
# subject = "str.format_map"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""str.format_map: format_map applies a float spec to a Fraction value through the canonical formatter"""
from fractions import Fraction

assert "{x:.3f}".format_map({"x": Fraction(5, 2)}) == "2.500"

print("format_map_formats_fraction_float_spec OK")
