# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "surface"
# case = "import_signal"
# subject = "signal"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""signal: import_signal (surface)."""
import signal

assert hasattr(signal, "signal")
print("import_signal OK")
