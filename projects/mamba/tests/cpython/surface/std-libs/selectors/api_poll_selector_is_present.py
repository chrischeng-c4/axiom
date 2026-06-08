# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "selectors"
# dimension = "surface"
# case = "api_poll_selector_is_present"
# subject = "selectors.PollSelector"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""selectors.PollSelector: api_poll_selector_is_present (surface)."""
import selectors

assert hasattr(selectors, "PollSelector")
print("api_poll_selector_is_present OK")
