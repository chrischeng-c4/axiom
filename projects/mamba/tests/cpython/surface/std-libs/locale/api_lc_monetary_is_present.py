# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "locale"
# dimension = "surface"
# case = "api_lc_monetary_is_present"
# subject = "locale.LC_MONETARY"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""locale.LC_MONETARY: api_lc_monetary_is_present (surface)."""
import locale

assert hasattr(locale, "LC_MONETARY")
print("api_lc_monetary_is_present OK")
