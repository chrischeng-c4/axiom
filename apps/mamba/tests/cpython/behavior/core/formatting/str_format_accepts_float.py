# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "formatting"
# dimension = "behavior"
# case = "str_format_accepts_float"
# subject = "str.format"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""str.format: str.format float formatting accepts a float with a precision specifier"""
# formatting syntax uses only Python built-ins

value = 2.5
assert "{:.3f}".format(value) == "2.500"

print("str_format_accepts_float OK")
