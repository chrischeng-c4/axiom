# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "surface"
# case = "api_pthread_sigmask_is_present"
# subject = "signal.pthread_sigmask"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""signal.pthread_sigmask: api_pthread_sigmask_is_present (surface)."""
import signal

assert hasattr(signal, "pthread_sigmask")
print("api_pthread_sigmask_is_present OK")
