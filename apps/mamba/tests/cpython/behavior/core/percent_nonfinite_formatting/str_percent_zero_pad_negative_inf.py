# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "percent_nonfinite_formatting"
# dimension = "behavior"
# case = "str_percent_zero_pad_negative_inf"
# subject = "str.percent_format"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""str.percent_format: string percent formatting preserves zero padding for negative infinity"""
value = float("-inf")
result = "%08.2f" % value
assert result == "-0000inf"
assert type(result) is str

print("str_percent_zero_pad_negative_inf OK")
