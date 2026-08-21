# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "surface"
# case = "raise_signal_is_callable"
# subject = "signal.raise_signal"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""signal.raise_signal: raise_signal_is_callable (surface)."""
import signal

assert callable(signal.raise_signal)
print("raise_signal_is_callable OK")
