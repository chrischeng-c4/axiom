# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "locale"
# dimension = "surface"
# case = "api_lc_collate_is_present"
# subject = "locale.LC_COLLATE"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""locale.LC_COLLATE: api_lc_collate_is_present (surface)."""
import locale

assert hasattr(locale, "LC_COLLATE")
print("api_lc_collate_is_present OK")
