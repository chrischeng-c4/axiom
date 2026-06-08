# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "surface"
# case = "api_sigprof_is_present"
# subject = "signal.SIGPROF"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""signal.SIGPROF: api_sigprof_is_present (surface)."""
import signal

assert hasattr(signal, "SIGPROF")
print("api_sigprof_is_present OK")
