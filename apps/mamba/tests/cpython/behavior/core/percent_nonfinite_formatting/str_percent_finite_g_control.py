# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "percent_nonfinite_formatting"
# dimension = "behavior"
# case = "str_percent_finite_g_control"
# subject = "str.percent_format"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""str.percent_format: finite old-style percent g formatting remains unchanged"""
result = "%.3g" % 2500.0
assert result == "2.5e+03"

print("str_percent_finite_g_control OK")
