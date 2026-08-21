# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "498"
# dimension = "behavior"
# case = "interpolates_name_and_format_spec"
# subject = "fstring.interpolation"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.interpolation: a bare name field and a name with a format spec interpolate: f'{x}' is '42' and f'{x:04d}' is '0042' for x = 42"""
# f-string interpolation is syntax; no import needed

x = 42
assert f"{x}" == "42"
assert f"{x:04d}" == "0042"

print("interpolates_name_and_format_spec OK")
