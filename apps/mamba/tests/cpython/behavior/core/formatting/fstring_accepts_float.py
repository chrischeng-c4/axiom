# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "formatting"
# dimension = "behavior"
# case = "fstring_accepts_float"
# subject = "fstring.float_format"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.float_format: f-string float formatting accepts a float with a precision specifier"""
# formatting syntax uses only Python built-ins

value = 2.5
assert f"{value:.3f}" == "2.500"

print("fstring_accepts_float OK")
