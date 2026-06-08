# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "surface"
# case = "api_sigwinch_is_present"
# subject = "signal.SIGWINCH"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""signal.SIGWINCH: api_sigwinch_is_present (surface)."""
import signal

assert hasattr(signal, "SIGWINCH")
print("api_sigwinch_is_present OK")
