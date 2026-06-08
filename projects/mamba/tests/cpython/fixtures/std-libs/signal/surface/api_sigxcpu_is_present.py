# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "surface"
# case = "api_sigxcpu_is_present"
# subject = "signal.SIGXCPU"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""signal.SIGXCPU: api_sigxcpu_is_present (surface)."""
import signal

assert hasattr(signal, "SIGXCPU")
print("api_sigxcpu_is_present OK")
