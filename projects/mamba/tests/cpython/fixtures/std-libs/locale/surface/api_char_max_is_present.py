# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "locale"
# dimension = "surface"
# case = "api_char_max_is_present"
# subject = "locale.CHAR_MAX"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""locale.CHAR_MAX: api_char_max_is_present (surface)."""
import locale

assert hasattr(locale, "CHAR_MAX")
print("api_char_max_is_present OK")
