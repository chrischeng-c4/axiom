# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "surface"
# case = "api_raise_signal_is_present"
# subject = "signal.raise_signal"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""signal.raise_signal: api_raise_signal_is_present (surface)."""
import signal

assert hasattr(signal, "raise_signal")
print("api_raise_signal_is_present OK")
