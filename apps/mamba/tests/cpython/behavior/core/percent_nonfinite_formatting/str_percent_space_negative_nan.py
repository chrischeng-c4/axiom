# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "percent_nonfinite_formatting"
# dimension = "behavior"
# case = "str_percent_space_negative_nan"
# subject = "str.percent_format"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""str.percent_format: string percent formatting applies a space sign to negative NaN"""
value = -float("nan")
result = "% 8.2f" % value
assert result == "     nan"
assert type(result) is str

print("str_percent_space_negative_nan OK")
