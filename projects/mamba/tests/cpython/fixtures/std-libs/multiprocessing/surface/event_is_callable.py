# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing"
# dimension = "surface"
# case = "event_is_callable"
# subject = "multiprocessing.Event"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""multiprocessing.Event: event_is_callable (surface)."""
import multiprocessing

assert callable(multiprocessing.Event)
print("event_is_callable OK")
