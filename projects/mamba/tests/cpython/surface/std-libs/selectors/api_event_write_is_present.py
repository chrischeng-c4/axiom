# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "selectors"
# dimension = "surface"
# case = "api_event_write_is_present"
# subject = "selectors.EVENT_WRITE"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""selectors.EVENT_WRITE: api_event_write_is_present (surface)."""
import selectors

assert hasattr(selectors, "EVENT_WRITE")
print("api_event_write_is_present OK")
