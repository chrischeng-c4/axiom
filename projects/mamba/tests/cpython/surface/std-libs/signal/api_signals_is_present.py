# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "surface"
# case = "api_signals_is_present"
# subject = "signal.Signals"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""signal.Signals: api_signals_is_present (surface)."""
import signal

assert hasattr(signal, "Signals")
print("api_signals_is_present OK")
