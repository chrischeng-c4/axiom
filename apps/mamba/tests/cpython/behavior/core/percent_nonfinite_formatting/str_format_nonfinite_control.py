# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "percent_nonfinite_formatting"
# dimension = "behavior"
# case = "str_format_nonfinite_control"
# subject = "str.format"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""str.format: modern str.format preserves non-finite formatting"""
value = float("-inf")
result = "{:.2E}".format(value)
assert result == "-INF"

print("str_format_nonfinite_control OK")
