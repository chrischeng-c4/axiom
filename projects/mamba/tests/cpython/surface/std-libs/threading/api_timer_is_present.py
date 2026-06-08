# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threading"
# dimension = "surface"
# case = "api_timer_is_present"
# subject = "threading.Timer"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""threading.Timer: api_timer_is_present (surface)."""
import threading

assert hasattr(threading, "Timer")
print("api_timer_is_present OK")
