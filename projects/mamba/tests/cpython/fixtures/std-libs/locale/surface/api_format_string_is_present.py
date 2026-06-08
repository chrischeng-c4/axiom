# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "locale"
# dimension = "surface"
# case = "api_format_string_is_present"
# subject = "locale.format_string"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""locale.format_string: api_format_string_is_present (surface)."""
import locale

assert hasattr(locale, "format_string")
print("api_format_string_is_present OK")
