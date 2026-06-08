# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "selectors"
# dimension = "surface"
# case = "api_kqueue_selector_is_present"
# subject = "selectors.KqueueSelector"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""selectors.KqueueSelector: api_kqueue_selector_is_present (surface)."""
import selectors

assert hasattr(selectors, "KqueueSelector")
print("api_kqueue_selector_is_present OK")
