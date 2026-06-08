# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "surface"
# case = "api_setitimer_is_present"
# subject = "signal.setitimer"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""signal.setitimer: api_setitimer_is_present (surface)."""
import signal

assert hasattr(signal, "setitimer")
print("api_setitimer_is_present OK")
