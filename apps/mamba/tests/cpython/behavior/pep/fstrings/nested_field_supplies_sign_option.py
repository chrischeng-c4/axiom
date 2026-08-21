# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "fstrings"
# dimension = "behavior"
# case = "nested_field_supplies_sign_option"
# subject = "fstring.format_spec"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.format_spec: a nested {} inside the spec supplies the sign option: with sign='+', f'{42:{sign}d}' is '+42'"""
# a nested field can supply any part of the format spec

sign = "+"
assert f"{42:{sign}d}" == "+42", f"sign format = {f'{42:{sign}d}'!r}"

print("nested_field_supplies_sign_option OK")
