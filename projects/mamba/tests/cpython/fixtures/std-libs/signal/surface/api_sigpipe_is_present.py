# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "surface"
# case = "api_sigpipe_is_present"
# subject = "signal.SIGPIPE"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""signal.SIGPIPE: api_sigpipe_is_present (surface)."""
import signal

assert hasattr(signal, "SIGPIPE")
print("api_sigpipe_is_present OK")
