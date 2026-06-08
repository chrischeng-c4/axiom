# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "surface"
# case = "sigterm_not_callable"
# subject = "signal.SIGTERM"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""signal.SIGTERM: sigterm_not_callable (surface)."""
import signal

assert not callable(signal.SIGTERM)
print("sigterm_not_callable OK")
