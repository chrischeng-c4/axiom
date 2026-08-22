# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "percent_nonfinite_formatting"
# dimension = "behavior"
# case = "fstring_nonfinite_control"
# subject = "f-string formatting"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""f-string formatting: modern f-string formatting preserves non-finite formatting"""
value = float("nan")
result = f"{value:.2g}"
assert result == "nan"

print("fstring_nonfinite_control OK")
