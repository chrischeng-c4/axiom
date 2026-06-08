# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "selectors"
# dimension = "surface"
# case = "api_selector_key_is_present"
# subject = "selectors.SelectorKey"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""selectors.SelectorKey: api_selector_key_is_present (surface)."""
import selectors

assert hasattr(selectors, "SelectorKey")
print("api_selector_key_is_present OK")
