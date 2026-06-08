# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "surface"
# case = "api_sigill_is_present"
# subject = "signal.SIGILL"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""signal.SIGILL: api_sigill_is_present (surface)."""
import signal

assert hasattr(signal, "SIGILL")
print("api_sigill_is_present OK")
