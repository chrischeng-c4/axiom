# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "surface"
# case = "api_sigsys_is_present"
# subject = "signal.SIGSYS"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""signal.SIGSYS: api_sigsys_is_present (surface)."""
import signal

assert hasattr(signal, "SIGSYS")
print("api_sigsys_is_present OK")
