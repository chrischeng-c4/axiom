# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "surface"
# case = "formatter_is_callable"
# subject = "string.Formatter"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""string.Formatter: formatter_is_callable (surface)."""
import string

assert callable(string.Formatter)
print("formatter_is_callable OK")
