# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "percent_nonfinite_formatting"
# dimension = "behavior"
# case = "str_percent_g_positive_inf"
# subject = "str.percent_format"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""str.percent_format: str percent g formatting renders positive inf without a bogus exponent"""
value = float("inf")
result = "%.2g" % value
assert result == "inf"
assert type(result) is str

print("str_percent_g_positive_inf OK")
