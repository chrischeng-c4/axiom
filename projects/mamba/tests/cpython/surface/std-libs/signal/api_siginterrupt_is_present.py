# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "surface"
# case = "api_siginterrupt_is_present"
# subject = "signal.siginterrupt"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""signal.siginterrupt: api_siginterrupt_is_present (surface)."""
import signal

assert hasattr(signal, "siginterrupt")
print("api_siginterrupt_is_present OK")
