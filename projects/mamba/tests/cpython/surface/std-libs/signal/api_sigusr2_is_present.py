# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "surface"
# case = "api_sigusr2_is_present"
# subject = "signal.SIGUSR2"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""signal.SIGUSR2: api_sigusr2_is_present (surface)."""
import signal

assert hasattr(signal, "SIGUSR2")
print("api_sigusr2_is_present OK")
