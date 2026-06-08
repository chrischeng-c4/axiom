# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "surface"
# case = "api_sigterm_is_present"
# subject = "signal.SIGTERM"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""signal.SIGTERM: api_sigterm_is_present (surface)."""
import signal

assert hasattr(signal, "SIGTERM")
print("api_sigterm_is_present OK")
