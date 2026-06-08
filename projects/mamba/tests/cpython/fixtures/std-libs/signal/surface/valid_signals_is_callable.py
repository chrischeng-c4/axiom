# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "surface"
# case = "valid_signals_is_callable"
# subject = "signal.valid_signals"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""signal.valid_signals: valid_signals_is_callable (surface)."""
import signal

assert callable(signal.valid_signals)
print("valid_signals_is_callable OK")
