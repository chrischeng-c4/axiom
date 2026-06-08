# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "surface"
# case = "api_getitimer_is_present"
# subject = "signal.getitimer"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""signal.getitimer: api_getitimer_is_present (surface)."""
import signal

assert hasattr(signal, "getitimer")
print("api_getitimer_is_present OK")
