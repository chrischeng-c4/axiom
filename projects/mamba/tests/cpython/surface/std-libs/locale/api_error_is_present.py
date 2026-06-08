# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "locale"
# dimension = "surface"
# case = "api_error_is_present"
# subject = "locale.Error"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""locale.Error: api_error_is_present (surface)."""
import locale

assert hasattr(locale, "Error")
print("api_error_is_present OK")
