# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "selectors"
# dimension = "surface"
# case = "api_default_selector_is_present"
# subject = "selectors.DefaultSelector"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""selectors.DefaultSelector: api_default_selector_is_present (surface)."""
import selectors

assert hasattr(selectors, "DefaultSelector")
print("api_default_selector_is_present OK")
