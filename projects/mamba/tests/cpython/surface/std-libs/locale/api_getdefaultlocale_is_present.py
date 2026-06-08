# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "locale"
# dimension = "surface"
# case = "api_getdefaultlocale_is_present"
# subject = "locale.getdefaultlocale"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""locale.getdefaultlocale: api_getdefaultlocale_is_present (surface)."""
import locale

assert hasattr(locale, "getdefaultlocale")
print("api_getdefaultlocale_is_present OK")
