# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threading"
# dimension = "surface"
# case = "event_has_is_set"
# subject = "threading.Event()"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""threading.Event(): event_has_is_set (surface)."""
import threading

assert hasattr(threading.Event(), "is_set")
print("event_has_is_set OK")
