# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "percent_nonfinite_formatting"
# dimension = "behavior"
# case = "str_percent_alternate_zero_precision_inf"
# subject = "str.percent_format"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""str.percent_format: string percent formatting ignores alternate decimal and exponent decoration for infinity"""
value = float("inf")
result = "%#.0e" % value
assert result == "inf"
assert type(result) is str

print("str_percent_alternate_zero_precision_inf OK")
