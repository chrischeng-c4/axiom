# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threading"
# dimension = "surface"
# case = "api_event_is_present"
# subject = "threading.Event"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""threading.Event: api_event_is_present (surface)."""
import threading

assert hasattr(threading, "Event")
print("api_event_is_present OK")
