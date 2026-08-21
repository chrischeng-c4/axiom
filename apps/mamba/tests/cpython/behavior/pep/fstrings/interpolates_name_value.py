# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "fstrings"
# dimension = "behavior"
# case = "interpolates_name_value"
# subject = "fstring.interpolation"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.interpolation: a bare name field interpolates its value: with x=10, f'{x}' is '10'"""
# f-string interpolation is syntax; no import needed

x = 10
assert f"{x}" == "10", f"basic = {f'{x}'!r}"

print("interpolates_name_value OK")
