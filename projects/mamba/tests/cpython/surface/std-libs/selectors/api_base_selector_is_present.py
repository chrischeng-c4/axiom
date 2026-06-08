# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "selectors"
# dimension = "surface"
# case = "api_base_selector_is_present"
# subject = "selectors.BaseSelector"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""selectors.BaseSelector: api_base_selector_is_present (surface)."""
import selectors

assert hasattr(selectors, "BaseSelector")
print("api_base_selector_is_present OK")
