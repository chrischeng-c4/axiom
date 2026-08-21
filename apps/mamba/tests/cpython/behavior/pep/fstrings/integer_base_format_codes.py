# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "fstrings"
# dimension = "behavior"
# case = "integer_base_format_codes"
# subject = "fstring.format_spec"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.format_spec: integer presentation codes render the base: f'{255:b}' is '11111111', f'{255:o}' is '377', f'{255:x}' is 'ff', f'{255:X}' is 'FF'"""
# b/o/x/X presentation types render an int in binary/octal/hex

assert f"{255:b}" == "11111111", "binary"
assert f"{255:o}" == "377", "octal"
assert f"{255:x}" == "ff", "hex lower"
assert f"{255:X}" == "FF", "hex upper"

print("integer_base_format_codes OK")
