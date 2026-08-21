# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "locale"
# dimension = "surface"
# case = "format_string_is_callable"
# subject = "locale.format_string"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""locale.format_string: format_string_is_callable (surface)."""
import locale

assert callable(locale.format_string)
print("format_string_is_callable OK")
