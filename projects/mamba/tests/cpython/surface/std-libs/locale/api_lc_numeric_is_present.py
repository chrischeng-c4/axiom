# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "locale"
# dimension = "surface"
# case = "api_lc_numeric_is_present"
# subject = "locale.LC_NUMERIC"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""locale.LC_NUMERIC: api_lc_numeric_is_present (surface)."""
import locale

assert hasattr(locale, "LC_NUMERIC")
print("api_lc_numeric_is_present OK")
