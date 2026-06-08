# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "surface"
# case = "api_sigcont_is_present"
# subject = "signal.SIGCONT"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""signal.SIGCONT: api_sigcont_is_present (surface)."""
import signal

assert hasattr(signal, "SIGCONT")
print("api_sigcont_is_present OK")
