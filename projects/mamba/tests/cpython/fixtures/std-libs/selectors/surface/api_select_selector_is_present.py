# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "selectors"
# dimension = "surface"
# case = "api_select_selector_is_present"
# subject = "selectors.SelectSelector"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""selectors.SelectSelector: api_select_selector_is_present (surface)."""
import selectors

assert hasattr(selectors, "SelectSelector")
print("api_select_selector_is_present OK")
