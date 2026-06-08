# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "surface"
# case = "api_sigfpe_is_present"
# subject = "signal.SIGFPE"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""signal.SIGFPE: api_sigfpe_is_present (surface)."""
import signal

assert hasattr(signal, "SIGFPE")
print("api_sigfpe_is_present OK")
