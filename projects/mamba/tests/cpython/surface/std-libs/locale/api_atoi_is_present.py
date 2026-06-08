# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "locale"
# dimension = "surface"
# case = "api_atoi_is_present"
# subject = "locale.atoi"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""locale.atoi: api_atoi_is_present (surface)."""
import locale

assert hasattr(locale, "atoi")
print("api_atoi_is_present OK")
