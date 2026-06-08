# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "fstrings"
# dimension = "surface"
# case = "str_has_format_dunder"
# subject = "str"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""str: str_has_format_dunder (surface)."""
# f-strings call __format__ on each replacement field

assert hasattr(str, "__format__")
print("str_has_format_dunder OK")
