# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "surface"
# case = "api_valid_signals_is_present"
# subject = "signal.valid_signals"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""signal.valid_signals: api_valid_signals_is_present (surface)."""
import signal

assert hasattr(signal, "valid_signals")
print("api_valid_signals_is_present OK")
