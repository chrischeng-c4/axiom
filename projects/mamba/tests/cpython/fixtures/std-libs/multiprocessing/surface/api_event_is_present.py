# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing"
# dimension = "surface"
# case = "api_event_is_present"
# subject = "multiprocessing.Event"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""multiprocessing.Event: api_event_is_present (surface)."""
import multiprocessing

assert hasattr(multiprocessing, "Event")
print("api_event_is_present OK")
