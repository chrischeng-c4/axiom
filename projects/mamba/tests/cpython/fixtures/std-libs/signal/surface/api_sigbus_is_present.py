# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "surface"
# case = "api_sigbus_is_present"
# subject = "signal.SIGBUS"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""signal.SIGBUS: api_sigbus_is_present (surface)."""
import signal

assert hasattr(signal, "SIGBUS")
print("api_sigbus_is_present OK")
