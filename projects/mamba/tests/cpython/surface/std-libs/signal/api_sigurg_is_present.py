# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "surface"
# case = "api_sigurg_is_present"
# subject = "signal.SIGURG"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""signal.SIGURG: api_sigurg_is_present (surface)."""
import signal

assert hasattr(signal, "SIGURG")
print("api_sigurg_is_present OK")
