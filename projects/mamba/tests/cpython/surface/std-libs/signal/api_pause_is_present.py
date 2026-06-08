# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "surface"
# case = "api_pause_is_present"
# subject = "signal.pause"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""signal.pause: api_pause_is_present (surface)."""
import signal

assert hasattr(signal, "pause")
print("api_pause_is_present OK")
