# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "surface"
# case = "api_sigio_is_present"
# subject = "signal.SIGIO"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""signal.SIGIO: api_sigio_is_present (surface)."""
import signal

assert hasattr(signal, "SIGIO")
print("api_sigio_is_present OK")
