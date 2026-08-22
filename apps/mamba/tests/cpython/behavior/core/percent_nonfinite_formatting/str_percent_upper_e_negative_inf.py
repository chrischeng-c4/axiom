# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "percent_nonfinite_formatting"
# dimension = "behavior"
# case = "str_percent_upper_e_negative_inf"
# subject = "str.percent_format"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""str.percent_format: str percent E formatting renders negative inf without a bogus exponent"""
value = float("-inf")
result = "%.2E" % value
assert result == "-INF"
assert type(result) is str

print("str_percent_upper_e_negative_inf OK")
