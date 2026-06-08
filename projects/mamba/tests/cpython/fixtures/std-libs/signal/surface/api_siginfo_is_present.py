# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "surface"
# case = "api_siginfo_is_present"
# subject = "signal.SIGINFO"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""signal.SIGINFO: api_siginfo_is_present (surface)."""
import signal

assert hasattr(signal, "SIGINFO")
print("api_siginfo_is_present OK")
