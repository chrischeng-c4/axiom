# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "surface"
# case = "api_sigvtalrm_is_present"
# subject = "signal.SIGVTALRM"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""signal.SIGVTALRM: api_sigvtalrm_is_present (surface)."""
import signal

assert hasattr(signal, "SIGVTALRM")
print("api_sigvtalrm_is_present OK")
