# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "surface"
# case = "api_sigusr1_is_present"
# subject = "signal.SIGUSR1"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""signal.SIGUSR1: api_sigusr1_is_present (surface)."""
import signal

assert hasattr(signal, "SIGUSR1")
print("api_sigusr1_is_present OK")
