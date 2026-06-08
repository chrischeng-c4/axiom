# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "surface"
# case = "api_sigabrt_is_present"
# subject = "signal.SIGABRT"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""signal.SIGABRT: api_sigabrt_is_present (surface)."""
import signal

assert hasattr(signal, "SIGABRT")
print("api_sigabrt_is_present OK")
