# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "surface"
# case = "signals_enum_exists"
# subject = "signal"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""signal: signals_enum_exists (surface)."""
import signal

assert hasattr(signal, "Signals")
print("signals_enum_exists OK")
