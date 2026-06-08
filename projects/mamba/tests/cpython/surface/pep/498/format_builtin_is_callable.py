# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "498"
# dimension = "surface"
# case = "format_builtin_is_callable"
# subject = "format"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""format: format_builtin_is_callable (surface)."""
# PEP 498 f-strings dispatch via the format() builtin / str.__format__

assert callable(format)
print("format_builtin_is_callable OK")
