# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "surface"
# case = "api_sigchld_is_present"
# subject = "signal.SIGCHLD"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""signal.SIGCHLD: api_sigchld_is_present (surface)."""
import signal

assert hasattr(signal, "SIGCHLD")
print("api_sigchld_is_present OK")
