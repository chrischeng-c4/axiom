# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "format_map"
# dimension = "behavior"
# case = "format_map_formats_float_control"
# subject = "str.format_map"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""str.format_map: format_map preserves ordinary float formatting"""
# formatting syntax uses only Python built-ins

assert "{x:.3f}".format_map({"x": 2.5}) == "2.500"

print("format_map_formats_float_control OK")
