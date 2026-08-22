# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "percent_nonfinite_formatting"
# dimension = "behavior"
# case = "str_percent_f_nan"
# subject = "str.percent_format"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""str.percent_format: str percent f formatting renders nan without a bogus exponent"""
value = float("nan")
result = "%.2f" % value
assert result == "nan"
assert type(result) is str

print("str_percent_f_nan OK")
