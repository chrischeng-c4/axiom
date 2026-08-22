# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "percent_nonfinite_formatting"
# dimension = "behavior"
# case = "format_function_nonfinite_control"
# subject = "builtins.format"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""builtins.format: modern format function preserves non-finite formatting"""
value = float("inf")
result = format(value, ".2f")
assert result == "inf"

print("format_function_nonfinite_control OK")
