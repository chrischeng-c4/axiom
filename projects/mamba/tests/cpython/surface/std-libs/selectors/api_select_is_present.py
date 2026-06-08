# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "selectors"
# dimension = "surface"
# case = "api_select_is_present"
# subject = "selectors.select"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""selectors.select: api_select_is_present (surface)."""
import selectors

assert hasattr(selectors, "select")
print("api_select_is_present OK")
