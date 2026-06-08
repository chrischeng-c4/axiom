# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "surface"
# case = "pause_is_callable"
# subject = "signal.pause"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""signal.pause: pause_is_callable (surface)."""
import signal

assert callable(signal.pause)
print("pause_is_callable OK")
