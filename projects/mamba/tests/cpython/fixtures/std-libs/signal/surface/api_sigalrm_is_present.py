# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "surface"
# case = "api_sigalrm_is_present"
# subject = "signal.SIGALRM"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""signal.SIGALRM: api_sigalrm_is_present (surface)."""
import signal

assert hasattr(signal, "SIGALRM")
print("api_sigalrm_is_present OK")
