# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "surface"
# case = "sigkill_not_callable"
# subject = "signal.SIGKILL"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""signal.SIGKILL: sigkill_not_callable (surface)."""
import signal

assert not callable(signal.SIGKILL)
print("sigkill_not_callable OK")
