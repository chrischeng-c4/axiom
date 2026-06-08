# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "surface"
# case = "api_itimer_virtual_is_present"
# subject = "signal.ITIMER_VIRTUAL"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""signal.ITIMER_VIRTUAL: api_itimer_virtual_is_present (surface)."""
import signal

assert hasattr(signal, "ITIMER_VIRTUAL")
print("api_itimer_virtual_is_present OK")
