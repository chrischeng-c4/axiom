# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "surface"
# case = "strsignal_is_callable"
# subject = "signal.strsignal"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""signal.strsignal: strsignal_is_callable (surface)."""
import signal

assert callable(signal.strsignal)
print("strsignal_is_callable OK")
