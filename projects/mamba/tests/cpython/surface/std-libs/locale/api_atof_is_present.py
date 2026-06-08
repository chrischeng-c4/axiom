# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "locale"
# dimension = "surface"
# case = "api_atof_is_present"
# subject = "locale.atof"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""locale.atof: api_atof_is_present (surface)."""
import locale

assert hasattr(locale, "atof")
print("api_atof_is_present OK")
