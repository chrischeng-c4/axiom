# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "formatting"
# dimension = "behavior"
# case = "percent_f_accepts_int"
# subject = "str.percent_format"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""str.percent_format: percent f formatting accepts an integer and renders three fractional digits"""
# formatting syntax uses only Python built-ins

value = 7
assert "%.3f" % value == "7.000"

print("percent_f_accepts_int OK")
