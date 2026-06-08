# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "locale"
# dimension = "surface"
# case = "api_resetlocale_is_present"
# subject = "locale.resetlocale"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""locale.resetlocale: api_resetlocale_is_present (surface)."""
import locale

assert hasattr(locale, "resetlocale")
print("api_resetlocale_is_present OK")
