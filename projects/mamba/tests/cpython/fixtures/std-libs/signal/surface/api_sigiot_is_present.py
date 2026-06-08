# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "surface"
# case = "api_sigiot_is_present"
# subject = "signal.SIGIOT"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""signal.SIGIOT: api_sigiot_is_present (surface)."""
import signal

assert hasattr(signal, "SIGIOT")
print("api_sigiot_is_present OK")
