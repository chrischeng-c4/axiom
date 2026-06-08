# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "surface"
# case = "api_itimer_real_is_present"
# subject = "signal.ITIMER_REAL"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""signal.ITIMER_REAL: api_itimer_real_is_present (surface)."""
import signal

assert hasattr(signal, "ITIMER_REAL")
print("api_itimer_real_is_present OK")
