# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "fstrings"
# dimension = "behavior"
# case = "float_precision_spec"
# subject = "fstring.format_spec"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.format_spec: a float precision spec rounds: f'{3.14159:.2f}' is '3.14'"""
# a presentation-type format spec controls float rendering

assert f"{3.14159:.2f}" == "3.14", f"float fmt = {f'{3.14159:.2f}'!r}"

print("float_precision_spec OK")
