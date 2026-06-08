# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "surface"
# case = "api_sigtrap_is_present"
# subject = "signal.SIGTRAP"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""signal.SIGTRAP: api_sigtrap_is_present (surface)."""
import signal

assert hasattr(signal, "SIGTRAP")
print("api_sigtrap_is_present OK")
