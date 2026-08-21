# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "surface"
# case = "sigint_not_callable"
# subject = "signal.SIGINT"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""signal.SIGINT: sigint_not_callable (surface)."""
import signal

assert not callable(signal.SIGINT)
print("sigint_not_callable OK")
