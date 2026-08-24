# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "percent_nonfinite_formatting"
# dimension = "behavior"
# case = "str_percent_plain_width_negative_inf"
# subject = "str.percent_format"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""str.percent_format: string percent formatting preserves plain width for negative infinity"""
value = float("-inf")
result = "%8.2f" % value
assert result == "    -inf"
assert type(result) is str

print("str_percent_plain_width_negative_inf OK")
