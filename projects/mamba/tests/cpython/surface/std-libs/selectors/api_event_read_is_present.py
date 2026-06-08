# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "selectors"
# dimension = "surface"
# case = "api_event_read_is_present"
# subject = "selectors.EVENT_READ"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""selectors.EVENT_READ: api_event_read_is_present (surface)."""
import selectors

assert hasattr(selectors, "EVENT_READ")
print("api_event_read_is_present OK")
