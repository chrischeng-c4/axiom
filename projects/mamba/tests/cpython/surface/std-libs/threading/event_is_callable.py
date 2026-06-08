# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threading"
# dimension = "surface"
# case = "event_is_callable"
# subject = "threading.Event"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""threading.Event: event_is_callable (surface)."""
import threading

assert callable(threading.Event)
print("event_is_callable OK")
