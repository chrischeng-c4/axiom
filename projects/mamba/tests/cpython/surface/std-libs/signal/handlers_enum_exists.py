# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "surface"
# case = "handlers_enum_exists"
# subject = "signal"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""signal: handlers_enum_exists (surface)."""
import signal

assert hasattr(signal, "Handlers")
print("handlers_enum_exists OK")
