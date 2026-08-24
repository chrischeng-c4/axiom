# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "percent_s_numeric_handles"
# dimension = "behavior"
# case = "percent_s_formats_int_control"
# subject = "str.percent_format"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""str.percent_format: primitive integer percent s formatting remains unchanged"""

assert "%s" % 42 == "42"

print("percent_s_formats_int_control OK")
