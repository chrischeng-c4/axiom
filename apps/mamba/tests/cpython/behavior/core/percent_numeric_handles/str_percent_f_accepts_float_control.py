# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "percent_numeric_handles"
# dimension = "behavior"
# case = "str_percent_f_accepts_float_control"
# subject = "str.percent_format"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""str.percent_format: ordinary string percent f formatting remains green with a float"""
assert "%.3f" % 2.5 == "2.500"

print("str_percent_f_accepts_float_control OK")
