# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "locale"
# dimension = "surface"
# case = "api_normalize_is_present"
# subject = "locale.normalize"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""locale.normalize: api_normalize_is_present (surface)."""
import locale

assert hasattr(locale, "normalize")
print("api_normalize_is_present OK")
