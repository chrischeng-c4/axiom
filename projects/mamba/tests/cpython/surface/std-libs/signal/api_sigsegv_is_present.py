# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "surface"
# case = "api_sigsegv_is_present"
# subject = "signal.SIGSEGV"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""signal.SIGSEGV: api_sigsegv_is_present (surface)."""
import signal

assert hasattr(signal, "SIGSEGV")
print("api_sigsegv_is_present OK")
