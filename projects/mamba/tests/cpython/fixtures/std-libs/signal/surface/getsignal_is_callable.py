# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "surface"
# case = "getsignal_is_callable"
# subject = "signal.getsignal"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""signal.getsignal: getsignal_is_callable (surface)."""
import signal

assert callable(signal.getsignal)
print("getsignal_is_callable OK")
