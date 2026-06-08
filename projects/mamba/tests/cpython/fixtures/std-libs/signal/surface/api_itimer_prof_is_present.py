# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "surface"
# case = "api_itimer_prof_is_present"
# subject = "signal.ITIMER_PROF"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""signal.ITIMER_PROF: api_itimer_prof_is_present (surface)."""
import signal

assert hasattr(signal, "ITIMER_PROF")
print("api_itimer_prof_is_present OK")
