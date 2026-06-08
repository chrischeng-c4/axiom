# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "surface"
# case = "api_sigemt_is_present"
# subject = "signal.SIGEMT"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""signal.SIGEMT: api_sigemt_is_present (surface)."""
import signal

assert hasattr(signal, "SIGEMT")
print("api_sigemt_is_present OK")
