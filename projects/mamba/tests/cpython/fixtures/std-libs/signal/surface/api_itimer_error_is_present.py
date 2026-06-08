# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "surface"
# case = "api_itimer_error_is_present"
# subject = "signal.ItimerError"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""signal.ItimerError: api_itimer_error_is_present (surface)."""
import signal

assert hasattr(signal, "ItimerError")
print("api_itimer_error_is_present OK")
