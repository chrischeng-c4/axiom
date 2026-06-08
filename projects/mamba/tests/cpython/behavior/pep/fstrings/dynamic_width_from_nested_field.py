# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "fstrings"
# dimension = "behavior"
# case = "dynamic_width_from_nested_field"
# subject = "fstring.format_spec"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.format_spec: a nested {} inside the spec supplies the width dynamically: with width=10, f"{'hi':^{width}}" centres to '    hi    '"""
# a replacement field may appear inside the format spec

width = 10
centered = f"{'hi':^{width}}"
assert centered == "    hi    ", f"dynamic width = {centered!r}"

print("dynamic_width_from_nested_field OK")
