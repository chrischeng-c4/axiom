# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "surface"
# case = "api_sigkill_is_present"
# subject = "signal.SIGKILL"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""signal.SIGKILL: api_sigkill_is_present (surface)."""
import signal

assert hasattr(signal, "SIGKILL")
print("api_sigkill_is_present OK")
