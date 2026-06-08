# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "surface"
# case = "api_sigstop_is_present"
# subject = "signal.SIGSTOP"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""signal.SIGSTOP: api_sigstop_is_present (surface)."""
import signal

assert hasattr(signal, "SIGSTOP")
print("api_sigstop_is_present OK")
