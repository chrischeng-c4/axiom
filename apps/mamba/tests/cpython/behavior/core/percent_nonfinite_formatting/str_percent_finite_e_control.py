# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "percent_nonfinite_formatting"
# dimension = "behavior"
# case = "str_percent_finite_e_control"
# subject = "str.percent_format"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""str.percent_format: finite old-style percent e formatting remains unchanged"""
result = "%.3e" % 2.5
assert result == "2.500e+00"

print("str_percent_finite_e_control OK")
