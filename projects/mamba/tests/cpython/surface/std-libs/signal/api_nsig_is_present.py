# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "surface"
# case = "api_nsig_is_present"
# subject = "signal.NSIG"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""signal.NSIG: api_nsig_is_present (surface)."""
import signal

assert hasattr(signal, "NSIG")
print("api_nsig_is_present OK")
