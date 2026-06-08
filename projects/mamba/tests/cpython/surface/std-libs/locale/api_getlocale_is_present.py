# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "locale"
# dimension = "surface"
# case = "api_getlocale_is_present"
# subject = "locale.getlocale"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""locale.getlocale: api_getlocale_is_present (surface)."""
import locale

assert hasattr(locale, "getlocale")
print("api_getlocale_is_present OK")
